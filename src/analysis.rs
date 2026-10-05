//! 目录大小分析。
//!
//! 从根路径出发递归统计每个条目的磁盘占用，构建出一棵
//! [`AnalysisItem`] 树，供输出层渲染。

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::file_info::FileInfo;

/// 分析结果树中的一个节点（文件或目录）。
pub struct AnalysisItem {
    /// 条目名称（不含父路径）。
    pub name: String,
    /// 该条目占用的磁盘字节数。
    pub disk_size: u64,
    /// 子条目列表；文件为 `None`。
    pub children: Option<Vec<AnalysisItem>>,
}

impl AnalysisItem {
    /// 递归分析 `path` 并返回其 [`AnalysisItem`] 树。
    ///
    /// - `apparent` 透传给 [`FileInfo::from_path`]，决定大小统计口径。
    /// - `root_dev` 为根路径所在卷标识，用于阻止跨越文件系统边界。
    ///
    /// 子目录通过 `rayon` 并行分析，结果按大小降序排列；无法读取的
    /// 子条目会被忽略而非中断整次分析。
    pub fn analyze(path: &Path, apparent: bool, root_dev: u64) -> Result<Self, Box<dyn Error>> {
        let name: String = path
            .file_name()
            .unwrap_or(OsStr::new("."))
            .to_string_lossy()
            .to_string();

        let file_info: FileInfo = FileInfo::from_path(path, apparent)?;

        match file_info {
            FileInfo::Directory { volume_id } => {
                // 跨文件系统（如挂载点）时停止下探。
                if volume_id != root_dev {
                    return Err("Filesystem boundary crossed.".into());
                }

                let sub_entries = fs::read_dir(path)?
                    .filter_map(Result::ok)
                    .collect::<Vec<_>>();

                let mut sub_items = sub_entries
                    .par_iter()
                    .filter_map(|entry| {
                        AnalysisItem::analyze(&entry.path(), apparent, root_dev).ok()
                    })
                    .collect::<Vec<_>>();

                sub_items.sort_unstable_by(|a, b| a.disk_size.cmp(&b.disk_size).reverse());

                Ok(AnalysisItem {
                    name,
                    disk_size: sub_items.iter().map(|di| di.disk_size).sum(),
                    children: Some(sub_items),
                })
            }
            FileInfo::File { size, .. } => Ok(AnalysisItem {
                name,
                disk_size: size,
                children: None,
            }),
        }
    }
}
