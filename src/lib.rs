use rayon::prelude::*;
use serde::Serialize;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use winapi::shared::winerror::NO_ERROR;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::fileapi::GetCompressedFileSizeW;
use winapi::um::fileapi::INVALID_FILE_SIZE;


#[derive(Serialize)]
pub struct AnalysisItem {
    pub name: String,
    pub disk_size: u64,
    pub children: Option<Vec<AnalysisItem>>,
}

impl AnalysisItem {
    pub fn analyze(path: &Path, apparent: bool, root_dev: u64) -> Result<Self, Box<dyn Error>> {
        let name: String = path
            .file_name()
            .unwrap_or(&OsStr::new("."))
            .to_string_lossy()
            .to_string();

        let file_info: FileInfo = FileInfo::from_path(path, apparent)?;

        match file_info {
            FileInfo::Directory { volume_id } => {
                if volume_id != root_dev {
                    return Err("Filesystem boundary crossed.".into());
                }

                let sub_entries = fs::read_dir(path)?.filter_map(Result::ok).collect::<Vec<_>>();

                let mut sub_items = sub_entries
                    .par_iter()
                    .filter_map(|entry| {
                        AnalysisItem::analyze(&entry.path(), apparent, root_dev).ok()
                    }).collect::<Vec<_>>();

                sub_items.sort_unstable_by(|a, b| a.disk_size.cmp(&b.disk_size).reverse());

                Ok(AnalysisItem { name, disk_size: sub_items.iter().map(|di| di.disk_size).sum(), children: Some(sub_items), }
                )
            }
            FileInfo::File { size, .. } => Ok(AnalysisItem {
                name,
                disk_size: size,
                children: None,
            }),
        }
    }
}

pub enum FileInfo {
    File { size: u64, volume_id: u64 },
    Directory { volume_id: u64 },
}

impl FileInfo {
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

    #[cfg(unix)]
    pub fn from_path(path: &Path, apparent: bool) -> Result<Self, Box<dyn Error>> {
        use std::os::unix::fs::MetadataExt;

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


pub fn compressed_size(path: &Path) -> Result<u64, Box<dyn Error>> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
    let mut high: u32 = 0;

    // TODO: Deal with max path size
    let low = unsafe { GetCompressedFileSizeW(wide.as_ptr(), &mut high) };

    if low == INVALID_FILE_SIZE {
        let err = get_last_error();
        if err != NO_ERROR {
            return Err(std::io::Error::last_os_error().into());
        }
    }

    Ok(u64::from(high) << 32 | u64::from(low))
}

fn get_last_error() -> u32 {
    unsafe { GetLastError() }
}
