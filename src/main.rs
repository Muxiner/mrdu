use std::env;
use std::error::Error;
use std::io;
use structopt::StructOpt;
use std::path::PathBuf;

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

    #[structopt(
        short = "p",
        default_value = "0.01",
        )]
    /// Threshold that determines if entry is worth being shown. 
    /// Between 0-100 % of dir size.
    min_percent: f64,

    #[structopt(short = "a")]
    /// Apparent size on disk
    /// 
    /// This would actually retrieve allocation size of files (AKA physical size on disk)
    apparent: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    // println!("Hello, world!");
    let test_args = Arguments::from_args();
    let current_dir = env::current_dir()?;
    let target_dir = test_args.target_dir.as_ref().unwrap_or(&current_dir);
    println!("current_dir: {}.", current_dir.display());
    println!("target_dir: {}", target_dir.display());
    Ok(())
}

/// 函数，磁盘分析结果
fn _show_disk_analyze_result() -> io::Result<()> {
    Ok(())
}

/// 函数，帮助信息
fn _show_help() -> io::Result<()> {
    Ok(())
}

/// 函数，磁盘分析结果 —— 单个项
fn _show_item_disk_analyze() -> io::Result<()> {
    Ok(())
}
