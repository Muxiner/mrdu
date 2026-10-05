//! 程序入口：解析参数、分析目标目录并渲染结果。

use atty::Stream;
use std::env;
use std::error::Error;
use structopt::StructOpt;
use termcolor::{BufferWriter, ColorChoice};

use mrdu::analysis::AnalysisItem;
use mrdu::args::Arguments;
use mrdu::file_info::FileInfo;
use mrdu::output::display_info::DisplayItemInfo;
use mrdu::output::show_disk_analyze_result;

fn main() -> Result<(), Box<dyn Error>> {
    let test_args = Arguments::from_args();
    // 未指定目标目录时使用当前工作目录。
    let current_dir = env::current_dir()?;
    let target_dir = test_args.target_dir.as_ref().unwrap_or(&current_dir);
    let file_info = FileInfo::from_path(target_dir, test_args.apparent)?;

    // 输出到终端时启用颜色，被重定向到文件/管道时禁用。
    let color_choice = if atty::is(Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };

    let stdout = BufferWriter::stdout(color_choice);
    let mut buffer = stdout.buffer();

    println!("\nAnalyzing: {}", target_dir.display());

    let start_time = std::time::Instant::now();
    let analysed = match file_info {
        FileInfo::Directory { volume_id } => {
            AnalysisItem::analyze(target_dir, test_args.apparent, volume_id)?
        }
        _ => return Err(format!("{} is not a directory!", target_dir.display()).into()),
    };
    show_disk_analyze_result(&analysed, &test_args, &DisplayItemInfo::new(), &mut buffer)?;
    stdout.print(&buffer)?;
    let elapsed_time = start_time.elapsed();
    println!("\nElapsed time: {:?}", elapsed_time);
    Ok(())
}
