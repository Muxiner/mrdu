//! 分析结果的终端渲染。
//!
//! 把 [`AnalysisItem`] 树按深度与占比阈值筛选后，绘制成带颜色和
//! 树形连接符的文本输出。同时提供字节数格式化与平台相关的辅助函数。

pub mod color;
pub mod display_info;
pub mod tree_shape;

use std::io;
use std::io::Write;
use termcolor::{Buffer, ColorSpec, WriteColor};

use crate::analysis::AnalysisItem;
use crate::args::Arguments;
use crate::output::color::COLOR_GRAY;
use crate::output::display_info::DisplayItemInfo;

/// 递归渲染分析结果树。
///
/// 仅渲染深度不超过 `config.max_depth`、且占父级比例大于
/// `config.min_percent` 的条目；每个子项通过 [`DisplayItemInfo`]
/// 携带缩进与前缀信息。
pub fn show_disk_analyze_result(
    item: &AnalysisItem,
    config: &Arguments,
    info: &DisplayItemInfo,
    buffer: &mut Buffer,
) -> io::Result<()> {
    show_disk_analyze_item(item, config, info, buffer)?;

    if info.dir_level < config.max_depth {
        if let Some(children) = &item.children {
            let children = children
                .iter()
                .map(|child| (child, size_fraction(child, item)))
                .filter(|&(_, occupied_size)| occupied_size > config.min_percent)
                .collect::<Vec<_>>();

            // 最后一个子项使用不同的连接符，单独处理。
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

/// 渲染单个条目，依次输出缩进、占比、大小、连接符与名称。
pub fn show_disk_analyze_item(
    item: &AnalysisItem,
    config: &Arguments,
    info: &DisplayItemInfo,
    buffer: &mut Buffer,
) -> io::Result<()> {
    // 缩进与树形连接符
    buffer.set_color(ColorSpec::new().set_fg(COLOR_GRAY))?;
    write!(buffer, "{}{}", info.prefix, info.display_prefix(true))?;
    // 占比
    buffer.set_color(ColorSpec::new().set_fg(info.display_color(false)))?;
    write!(
        buffer,
        " {} ",
        format_args!(
            "{:1$.2$}%",
            info.occupied_size,
            config.decimal_num + 3,
            config.decimal_num
        )
    )?;
    // 磁盘大小
    buffer.set_color(ColorSpec::new().set_fg(info.display_color(true)))?;
    write!(buffer, "[{}]", convert_to_bytes(item.disk_size as f64),)?;
    // 箭头
    buffer.set_color(ColorSpec::new().set_fg(COLOR_GRAY))?;
    write!(buffer, " {} ", tree_shape::SPACING)?;
    // 名称
    buffer.reset()?;
    writeln!(buffer, "{}", item.name)?;
    Ok(())
}

/// 计算 `child` 占 `parent` 磁盘大小的百分比（0-100）。
pub fn size_fraction(child: &AnalysisItem, parent: &AnalysisItem) -> f64 {
    100.0 * (child.disk_size as f64 / parent.disk_size as f64)
}

/// 把字节数格式化为带单位的人类可读字符串（以 1000 为进制）。
///
/// 改写自 `pretty_bytes::converter::convert`。
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

/// 读取 Windows 上文件的压缩后实际大小（`GetCompressedFileSizeW`）。
#[cfg(windows)]
pub fn compressed_size(path: &std::path::Path) -> Result<u64, Box<dyn std::error::Error>> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use winapi::shared::winerror::NO_ERROR;
    use winapi::um::fileapi::{GetCompressedFileSizeW, INVALID_FILE_SIZE};

    // GetCompressedFileSizeW 是宽字符版本，需要以 NUL 结尾的 UTF-16 路径。
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
    let mut high: u32 = 0;
    let low = unsafe { GetCompressedFileSizeW(wide.as_ptr(), &mut high) };

    if low == INVALID_FILE_SIZE {
        let err = get_last_error();
        if err != NO_ERROR {
            return Err(std::io::Error::last_os_error().into());
        }
    }

    Ok(u64::from(high) << 32 | u64::from(low))
}

/// 获取 Windows 最近一次错误码。
#[cfg(windows)]
pub fn get_last_error() -> u32 {
    use winapi::um::errhandlingapi::GetLastError;
    unsafe { GetLastError() }
}
