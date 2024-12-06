//! # The Command-Line Arguments

use std::path::PathBuf;

/// The HTTP Server's command line arguments
#[derive(Debug)]
pub struct Args {
    /// The directory where the files are stored, as an absolute path
    pub dir: PathBuf,
}

pub fn cli_args(args: &[String]) -> Option<Args> {
    if args.len() == 1 {
        None
    } else if args[1] == "--directory" {
        if args.len() == 2 {
            None
        } else {
            Some(Args {
                dir: PathBuf::from(&args[2]),
            })
        }
    } else {
        None
    }
}
