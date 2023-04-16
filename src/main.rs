use atty::Stream;
#[allow(unused_imports)]
use mrdu::{AnalysisItem, FileInfo};
use std::env;
use std::error::Error;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;
use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};
use pretty_bytes::converter::convert as pretty_bytes;

#[allow(dead_code)]
const INDENT_COLOR: Option<Color> = Some(Color::Rgb(75, 75, 75));

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
    #[structopt(parse(from_os_str))]
    /// Directory that needs to be analyzed.
    /// Default: current path
    target_dir: Option<PathBuf>,

    #[structopt(short = "d", default_value = "2")]
    /// Maximum recursion depth in directory.
    max_depth: usize,

    #[structopt(short = "p", default_value = "0.01")]
    /// Threshold that determines if entry is worth being shown.
    /// Between 0-100 % of dir size.
    min_percent: f64,

    #[structopt(short = "a")]
    /// Apparent size on disk
    ///
    /// This would actually retrieve allocation size of files (AKA physical size on disk)
    apparent: bool,
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
            dir_level: 1,
            is_last: true,
            indents_size: String::new(),
        }
    }

    fn add_item(&self, occupied_size: f64) -> Self {
        Self {
            occupied_size,
            dir_level: self.dir_level + 1,
            is_last: false,
            indents_size: self.indents_size.clone() + &self.indents_size + &String::from(" "),
        }
    }

    /// may be could become add_item(&self, occupied: f63, is_last: bool) -> Self {}
    fn add_last_item(&self, occupied_size: f64) -> Self {
        Self {
            occupied_size,
            dir_level: self.dir_level + 1,
            is_last: true,
            indents_size: self.indents_size.clone() + &self.indents_size + &String::from(" "),
        }
    }

    fn display_indent(&self) -> &'static str {
        if self.is_last {
            " "
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

    fn display_color(&self) -> Option<Color> {
        if self.dir_level == 0 {
            Some(Color::Green)
        } else if self.dir_level == 1 {
            Some(Color::Green)
        } else if self.dir_level == 2 {
            Some(Color::Green)
        } else if self.occupied_size >= 50.0 {
            Some(Color::Red)
        } else if self.occupied_size >= 10.0 && self.occupied_size < 50.0 {
            Some(Color::Yellow)
        } else {
            Some(Color::White)
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
    show_item_disk_analyze(item, &info, buffer)?;

    if info.dir_level < config.max_depth {
        if let Some(children) = &item.children {
            let children = children
            .iter()
            .map(|child| (child, size_fraction(child, item)))
            .filter(|&(_, occupied_size)| occupied_size > config.min_percent)
            .collect::<Vec<_>>();

            if let Some((last_chlid, children)) = children.split_last() {
                for &(child, occupied_size) in children.iter() {
                    show_disk_analyze_result(child, config, &info.add_item(occupied_size), buffer)?;
                }
                let &(child, occupied_size) = last_chlid;
                show_disk_analyze_result(child, config, &info.add_last_item(occupied_size), buffer)?;
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
fn show_item_disk_analyze(item: &AnalysisItem, info: &DisplayItemInfo, buffer: &mut Buffer) -> io::Result<()> {
    // Indentation
    buffer.set_color(ColorSpec::new().set_fg(INDENT_COLOR))?;
    write!(buffer, "{}{}", info.indents_size, info.display_prefix())?;
    // Percentage
    buffer.set_color(ColorSpec::new().set_fg(info.display_color()))?;
    write!(buffer, " {} ", format!("{:.2}%", info.occupied_size))?;
    // Disk size
    buffer.reset()?;
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