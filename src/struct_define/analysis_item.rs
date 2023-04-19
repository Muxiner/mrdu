use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::struct_define::file_info::FileInfo;

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
            .unwrap_or(OsStr::new("."))
            .to_string_lossy()
            .to_string();

        let file_info: FileInfo = FileInfo::from_path(path, apparent)?;

        match file_info {
            FileInfo::Directory { volume_id } => {
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
