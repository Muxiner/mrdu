use std::io;
use std::io::Write;
use termcolor::{Buffer, ColorSpec, WriteColor};

use crate::struct_define::analysis_item::AnalysisItem;
use crate::struct_define::config::Arguments;
use crate::struct_define::display_color::COLOR_GRAY;
use crate::struct_define::display_info::DisplayItemInfo;
use crate::struct_define::tree_shape;

#[cfg(windows)]
use std::error::Error;
#[cfg(windows)]
use std::path::Path;

/// 函数，磁盘分析结果
pub fn show_disk_analyze_result(
    item: &AnalysisItem,
    config: &Arguments,
    info: &DisplayItemInfo,
    buffer: &mut Buffer,
) -> io::Result<()> {
    show_disk_analyze_item(item, &config, &info, buffer)?;

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
pub fn show_disk_analyze_item(
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
    write!(buffer, "[{}]", convert_to_bytes(item.disk_size as f64),)?;
    // write!(buffer, "[{}]", (item.disk_size as f64).to_string() )?;
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

// pretty_bytes::converter::convert 据此修改修改
pub fn convert_to_bytes(num: f64) -> String {
    use std::cmp;
    let negative = if num.is_sign_positive() { "" } else { "-" };
    let num = num.abs();
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    if num < 1_f64 {
        return format!("{}{} {}", negative, num, "B");
    }
    let delimiter = 1000_f64;
    let exponent = cmp::min(
        (num.ln() / delimiter.ln()).floor() as i32,
        (units.len() - 1) as i32,
    );
    let pretty_bytes = format!("{:.2}", num / delimiter.powi(exponent))
        .parse::<f64>()
        .unwrap()
        * 1_f64;
    let unit = units[exponent as usize];
    format!("{}{} {}", negative, pretty_bytes, unit)
}

#[cfg(windows)]
pub fn compressed_size(path: &Path) -> Result<u64, Box<dyn Error>> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use winapi::shared::winerror::NO_ERROR;
    use winapi::um::fileapi::{GetCompressedFileSizeW, INVALID_FILE_SIZE};
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
    let mut high: u32 = 0;
    // use std::ptr::null_mut;
    // use winapi::um::fileapi::{CreateFileW, GetFileSize, OPEN_EXISTING};
    // use winapi::um::winnt::{GENERIC_READ, FILE_SHARE_READ};
    // use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    // let wide: Vec<u16> = path
    //     .as_os_str()
    //     .encode_wide()
    //     .chain(Some(0).into_iter())
    //     .collect();
    // let handle = unsafe {
    //     CreateFileW(
    //         wide.as_ptr(),
    //         GENERIC_READ,
    //         FILE_SHARE_READ,
    //         null_mut(),
    //         OPEN_EXISTING,
    //         0,
    //         null_mut(),
    //     )
    // };
    //
    // if handle == INVALID_HANDLE_VALUE {
    //     // handle 创建失败
    //     let err = get_last_error();
    //     if err != NO_ERROR {
    //         return Err(std::io::Error::last_os_error().into());
    //     }
    // }
    // let low = unsafe { GetFileSize(handle, &mut high) };

    // GetCompressedFileSizeW 是宽字符版本的函数，用于操作 Unicode 字符集的字符串。
    let low = unsafe { GetCompressedFileSizeW(wide.as_ptr(), &mut high) };

    if low == INVALID_FILE_SIZE {
        let err = get_last_error();
        if err != NO_ERROR {
            return Err(std::io::Error::last_os_error().into());
        }
    }

    Ok(u64::from(high) << 32 | u64::from(low))
}

#[cfg(windows)]
pub fn get_last_error() -> u32 {
    use winapi::um::errhandlingapi::GetLastError;
    unsafe { GetLastError() }
}
