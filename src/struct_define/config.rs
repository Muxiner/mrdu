use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "mrdu", about = "A simple command line disk analysis tool.")]
pub struct Arguments {
    /// Directory that needs to be analyzed
    /// [default: current path]
    #[structopt(parse(from_os_str))]
    pub target_dir: Option<PathBuf>,

    /// Maximum recursion depth in directory.
    #[structopt(short = "d", long = "max-depth", default_value = "2")]
    pub max_depth: usize,

    /// Threshold that determines if entry is worth being shown.
    /// Between 0-100%.
    #[structopt(short = "p", long = "min-percent", default_value = "10")]
    pub min_percent: f64,

    /// Apparent size on disk
    // This would actually retrieve allocation size of files (AKA physical size on disk)
    #[structopt(short = "a", long = "apparent")]
    pub apparent: bool,

    /// Number of decimal places
    // The number of decimal places occupied by files or folders.
    #[structopt(short = "n", long = "precision", default_value = "2")]
    pub decimal_num: usize,
}
