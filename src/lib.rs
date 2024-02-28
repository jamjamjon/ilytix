mod cli;
mod deduplicator;
mod image_files;
mod sanitizer;
mod utils;

pub use cli::{Cli, Task};
pub use deduplicator::DeDuplicator;
pub use image_files::ImageFiles;
pub use sanitizer::Sanitizer;
pub use utils::*;

const LOGGER: Logger = Logger;
const SAVEOUT_INCORRECT: &str = "Incorrect";
const SAVEOUT_RECTIFIED: &str = "Rectified";
const SAVEOUT_VALID: &str = "Intact";
const SAVEOUT_DEPRECATED: &str = "Deprecated Or Unsupported";
const SAVEOUT_DUPLICATED: &str = "Duplicated";
const SAVEOUT_CURATED: &str = "Curated";
const SAVEOUT_FILTERED: &str = "Filtered";
