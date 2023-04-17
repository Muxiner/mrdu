use std::error::Error;
use std::io;
use std::io::Write;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use termcolor::{Buffer, ColorSpec, WriteColor};
use pretty_bytes::converter::convert;
use winapi::shared::winerror::NO_ERROR;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::fileapi::{GetCompressedFileSizeW, INVALID_FILE_SIZE};

use crate::struct_define::analysis_item::AnalysisItem;
use crate::struct_define::config::Arguments;
use crate::struct_define::display_color::COLOR_GRAY;
use crate::struct_define::display_info::DisplayItemInfo;
use crate::struct_define::tree_shape;

/// 函数，磁盘分析结果
pub fn show_disk_analyze_result(
    item: &AnalysisItem,
    config: &Arguments,
    info: &DisplayItemInfo,
    buffer: &mut Buffer,
) -> io::Result<()> {
    show_item_disk_analyze(item, &config, &info, buffer)?;

    if info.dir_level < config.max_depth {
        if let Some(children) = &item.children {
            let children = children
                .iter()
                .map(|child| (child, size_fraction(child, item)))
                .filter(|&(_, occupied_size)| occupied_size > config.min_percent)
                .collect::<Vec<_>>();

            if let Some((last_child, children)) = children.split_last() {
                for &(child, occupied_size) in children.iter() {
                    show_disk_analyze_result(
                        child,
                        config,
                        &info.add_item(occupied_size, false),
                        buffer,
                    )?;
                }
                let &(child, occupied_size) = last_child;
                show_disk_analyze_result(
                    child,
                    config,
                    &info.add_item(occupied_size, true),
                    buffer,
                )?;
            }
        }
    }
    Ok(())
}

/// 函数，帮助信息
pub fn _show_help() -> io::Result<()> {
    Ok(())
}

/// 函数，磁盘分析结果 —— 单个项
pub fn show_item_disk_analyze(
    item: &AnalysisItem,
    config: &Arguments,
    info: &DisplayItemInfo,
    buffer: &mut Buffer,
) -> io::Result<()> {
    // Indentation
    buffer.set_color(ColorSpec::new().set_fg(COLOR_GRAY))?;
    write!(
        buffer,
        "{}{}",
        info.indents_size,
        info.display_prefix_indent(true)
    )?;
    // Percentage
    buffer.set_color(ColorSpec::new().set_fg(info.display_color(false)))?;
    write!(
        buffer,
        " {} ",
        format!(
            "{:1$.2$}%",
            info.occupied_size,
            config.decimal_num + 3,
            config.decimal_num
        )
    )?;
    // Disk size
    buffer.set_color(ColorSpec::new().set_fg(info.display_color(true)))?;
    write!(buffer, "[{}]", convert(item.disk_size as f64),)?;
    // Arrow
    buffer.set_color(ColorSpec::new().set_fg(COLOR_GRAY))?;
    write!(buffer, " {} ", tree_shape::SPACING)?;
    // Name
    buffer.reset()?;
    writeln!(buffer, "{}", item.name)?;
    Ok(())
}

pub fn size_fraction(child: &AnalysisItem, parent: &AnalysisItem) -> f64 {
    100.0 * (child.disk_size as f64 / parent.disk_size as f64)
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

pub fn get_last_error() -> u32 {
    unsafe { GetLastError() }
}