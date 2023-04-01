use std::error::Error;
use std::env;
use std::io;
use structopt::StructOpt;

// 模块，终端输出树形结构视觉效果
pub mod tree_shape {

}

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example command line program.")]
struct Arguments {
    #[structopt(short = "d", long)]
    /// args one
    _args_one: i8,

    #[structopt(short ="a", long)]
    /// args_two
    _atgs_two: i8,

    #[structopt(short = "s", long)]
    /// args_three
    _args_three: i8,

    #[structopt(short = "f", long)]
    /// args_four
    _arg_four: i8,
}

fn main() -> Result<(), Box<dyn Error>>{
    println!("Hello, world!");
    let test_args = Arguments::from_args();
    // let current_dir = env::current_dir()?;
    // println!("current_dir: {}.", current_dir.display());
    // println!("{:?}", test_args);
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
