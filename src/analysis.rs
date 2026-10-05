//! 目录大小分析。
//!
//! 从根路径出发递归统计每个条目的磁盘占用，构建出一棵
//! [`AnalysisItem`] 树，供输出层渲染。

use std::error::Error;
use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::thread;

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
    /// 仅对根路径的直接子项做一次并行划分（线程数不超过 CPU 核数），
    /// 更深层递归串行执行，从而把总线程数控制在有限范围内。结果按大小
    /// 降序排列；无法读取的子条目会被忽略而非中断整次分析。
    pub fn analyze(path: &Path, apparent: bool, root_dev: u64) -> Result<Self, Box<dyn Error>> {
        Self::build(path, apparent, root_dev, true)
    }

    /// 核心递归实现；`parallel` 决定本层是否对子项做并行划分。
    fn build(
        path: &Path,
        apparent: bool,
        root_dev: u64,
        parallel: bool,
    ) -> Result<Self, Box<dyn Error>> {
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

                let mut sub_items = if parallel {
                    analyze_parallel(&sub_entries, apparent, root_dev)
                } else {
                    analyze_sequential(&sub_entries, apparent, root_dev)
                };

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

/// 串行分析一批子项，忽略分析失败的条目。
fn analyze_sequential(entries: &[DirEntry], apparent: bool, root_dev: u64) -> Vec<AnalysisItem> {
    entries
        .iter()
        .filter_map(|entry| AnalysisItem::build(&entry.path(), apparent, root_dev, false).ok())
        .collect()
}

/// 按 CPU 核数把子项切成若干块，每块起一个作用域线程串行递归。
///
/// 由于块内递归不再并行，整次分析的并发线程数受 CPU 核数约束。
fn analyze_parallel(entries: &[DirEntry], apparent: bool, root_dev: u64) -> Vec<AnalysisItem> {
    let workers = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(entries.len());
    if workers <= 1 {
        return analyze_sequential(entries, apparent, root_dev);
    }

    let chunk_size = (entries.len() + workers - 1) / workers;
    thread::scope(|scope| {
        let handles = entries
            .chunks(chunk_size)
            .map(|chunk| scope.spawn(move || analyze_sequential(chunk, apparent, root_dev)))
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect()
    })
}
