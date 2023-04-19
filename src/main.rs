use atty::Stream;
use std::env;
use std::error::Error;
use structopt::StructOpt;
use termcolor::{BufferWriter, ColorChoice};

use mrdu::methods::show_disk_analyze_result;
use mrdu::struct_define::analysis_item::AnalysisItem;
use mrdu::struct_define::config::Arguments;
use mrdu::struct_define::display_info::DisplayItemInfo;
use mrdu::struct_define::file_info::FileInfo;

fn main() -> Result<(), Box<dyn Error>> {
    let test_args = Arguments::from_args();
    let current_dir = env::current_dir()?;
    let target_dir = test_args.target_dir.as_ref().unwrap_or(&current_dir);
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
