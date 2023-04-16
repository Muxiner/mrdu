use atty::Stream;
#[allow(unused_imports)]
use mrdu::{AnalysisItem, FileInfo};
use pretty_bytes::converter::convert as pretty_bytes;
use std::env;
use std::error::Error;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;
use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

#[allow(dead_code)]
const INDENT_COLOR: Option<Color> = Some(Color::Rgb(147, 147, 147));

// 模块，终端输出树形结构视觉效果
pub mod tree_shape {
    pub const SPACING: &str = "──";
    pub const BRANCH: &str = "│";
    pub const LEAF: &str = "├──";
    pub const _LEAF_WITH_BRANCH: &str = "├─┬";
    pub const LAST_LEAF: &str = "└──";
    pub const _LAST_LEAF_WITH_BRANCH: &str = "└─┬";
}

#[derive(Debug, StructOpt)]
#[structopt(name = "mrdu", about = "A simple command line disk analysis tool.")]
struct Arguments {
    /// Directory that needs to be analyzed
    /// [default: current path]
    #[structopt(parse(from_os_str))]
    target_dir: Option<PathBuf>,

    /// Maximum recursion depth in directory.
    #[structopt(short = "d", long = "max-depth", default_value = "2")]
    max_depth: usize,

    /// Threshold that determines if entry is worth being shown.
    /// Between 0-100%.
    #[structopt(short = "p", long = "min-percent", default_value = "0.01")]
    min_percent: f64,

    /// Apparent size on disk
    // This would actually retrieve allocation size of files (AKA physical size on disk)
    #[structopt(short = "a", long = "apparent")]
    apparent: bool,

    /// Number of decimal places
    // The number of decimal places occupied by files or folders.
    #[structopt(short = "n", long = "precision", default_value = "2")]
    decimal_num: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DisplayItemInfo {
    occupied_size: f64,
    dir_level: usize,
    is_last: bool,
    indents_size: String,
}

#[allow(dead_code)]
impl DisplayItemInfo {
    fn new() -> Self {
        Self {
            occupied_size: 100.0,
            dir_level: 0,
            is_last: true,
            indents_size: String::new(),
        }
    }

    fn add_item(&self, occupied_size: f64, is_last: bool) -> Self {
        Self {
            occupied_size,
            dir_level: self.dir_level + 1,
            is_last,
            indents_size: self.indents_size.clone() + self.display_prefix_indent(false) + &String::from("  "),
        }
    }

    fn display_prefix_indent(&self, is_prefix: bool) -> &'static str {
        match self.is_last {
            true => match is_prefix {
                true => tree_shape::LAST_LEAF,
                false => "  ",
            },
            false => match is_prefix {
                true => tree_shape::LEAF,
                false => tree_shape::BRANCH,
            }
        }
    }
    fn display_indent(&self) -> &'static str {
        if self.is_last {
            "  "
        } else {
            tree_shape::BRANCH
        }
    }

    fn display_prefix(&self) -> &'static str {
        if self.is_last {
            tree_shape::LAST_LEAF
        } else {
            tree_shape::LEAF
        }
    }

    fn display_color(&self, is_disk_size: bool) -> Option<Color> {
        let darken = |x: u8| (x as f32 * 0.5).round() as u8;
        let get_color = |r: u8, g: u8, b: u8| {
            if is_disk_size {
                Color::Rgb(darken(r), darken(g), b)
            } else {
                Color::Rgb(r, g, b)
            }
        };
        match self.dir_level {
            // Analyzed root directory, Purple
            0 => Some(get_color(250, 250, 250)),
            // Directories or files that occupied >= 50%, Red
            _ if self.occupied_size >= 50.0 => Some(get_color(255, 100, 100)),
            // Directories or files that occupied < 50.0% && >= 10.0%, Yellow
            _ if self.occupied_size >= 10.0 && self.occupied_size < 50.0 => {
                Some(get_color(255, 222, 72))
            }
            // Directories or files that occupied < 10.0%, Green
            _ => Some(get_color(100, 255, 90)),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // println!("Hello, world!");
    let test_args = Arguments::from_args();
    let current_dir = env::current_dir()?;
    let target_dir = test_args.target_dir.as_ref().unwrap_or(&current_dir);
    // println!("current_dir: {}", current_dir.display());
    // println!("target_dir: {}", target_dir.display());
    let file_info = FileInfo::from_path(&target_dir, test_args.apparent)?;

    let color_choice = if atty::is(Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };

    let stdout = BufferWriter::stdout(color_choice);
    let mut buffer = stdout.buffer();

    println!("\nAnalyzing: {}\n", target_dir.display());

    let analysed = match file_info {
        FileInfo::Directory { volume_id } => {
            AnalysisItem::analyze(&target_dir, test_args.apparent, volume_id)?
        }
        _ => return Err(format!("{} is not a directory!", target_dir.display()).into()),
    };

    show_disk_analyze_result(&analysed, &test_args, &DisplayItemInfo::new(), &mut buffer)?;
    stdout.print(&buffer)?;
    Ok(())
}

/// 函数，磁盘分析结果
fn show_disk_analyze_result(
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

            if let Some((last_chlid, children)) = children.split_last() {
                for &(child, occupied_size) in children.iter() {
                    show_disk_analyze_result(child, config, &info.add_item(occupied_size, false), buffer)?;
                }
                let &(child, occupied_size) = last_chlid;
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
fn _show_help() -> io::Result<()> {
    Ok(())
}

/// 函数，磁盘分析结果 —— 单个项
fn show_item_disk_analyze(
    item: &AnalysisItem,
    config: &Arguments,
    info: &DisplayItemInfo,
    buffer: &mut Buffer,
) -> io::Result<()> {
    // Indentation
    buffer.set_color(ColorSpec::new().set_fg(INDENT_COLOR))?;
    write!(buffer, "{}{}", info.indents_size, info.display_prefix_indent(true))?;
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
    write!(buffer, "[{}]", pretty_bytes(item.disk_size as f64),)?;
    // Arrow
    buffer.set_color(ColorSpec::new().set_fg(INDENT_COLOR))?;
    write!(buffer, " {} ", tree_shape::SPACING)?;
    // Name
    buffer.reset()?;
    writeln!(buffer, "{}", item.name)?;
    Ok(())
}

fn size_fraction(child: &AnalysisItem, parent: &AnalysisItem) -> f64 {
    100.0 * (child.disk_size as f64 / parent.disk_size as f64)
}
