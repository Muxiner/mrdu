//! 文件/目录的元信息读取。
//!
//! 按平台分别实现 [`FileInfo::from_path`]，屏蔽 Unix 与 Windows 在
//! 大小统计、卷标识上的差异，为上层分析提供统一入口。

use std::error::Error;
use std::path::Path;

#[cfg(windows)]
use crate::output::compressed_size;

/// 一个路径的元信息：文件（含大小）或目录。
pub enum FileInfo {
    /// 普通文件，`size` 为字节数，`volume_id` 为所在卷标识。
    File { size: u64, volume_id: u64 },
    /// 目录，`volume_id` 为所在卷标识。
    Directory { volume_id: u64 },
}

impl FileInfo {
    /// 读取 `path` 的元信息。
    ///
    /// `apparent` 为 `true` 时返回文件在磁盘上的**实际分配**大小，
    /// 否则返回逻辑大小。
    #[cfg(windows)]
    pub fn from_path(path: &Path, apparent: bool) -> Result<Self, Box<dyn Error>> {
        use winapi_util::{file, Handle};
        const FILE_ATTRIBUTE_DIRECTORY: u64 = 0x10;

        let h = Handle::from_path_any(path)?;
        let md = file::information(h)?;

        if md.file_attributes() & FILE_ATTRIBUTE_DIRECTORY != 0 {
            Ok(FileInfo::Directory {
                volume_id: md.volume_serial_number(),
            })
        } else {
            let size = if apparent {
                compressed_size(path)?
            } else {
                md.file_size()
            };
            Ok(FileInfo::File {
                size,
                volume_id: md.volume_serial_number(),
            })
        }
    }

    /// 读取 `path` 的元信息。
    ///
    /// `apparent` 为 `true` 时返回文件在磁盘上的**实际分配**大小
    /// （`st_blocks * 512`），否则返回逻辑大小（`st_size`）。
    #[cfg(unix)]
    pub fn from_path(path: &Path, apparent: bool) -> Result<Self, Box<dyn Error>> {
        use std::os::unix::fs::MetadataExt;

        // 使用 symlink_metadata 避免跟随符号链接。
        let md = path.symlink_metadata()?;
        if md.is_dir() {
            Ok(FileInfo::Directory {
                volume_id: md.dev(),
            })
        } else {
            let size = if apparent {
                md.blocks() * 512
            } else {
                md.len()
            };
            Ok(FileInfo::File {
                size,
                volume_id: md.dev(),
            })
        }
    }
}
