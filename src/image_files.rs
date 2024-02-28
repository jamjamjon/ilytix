use crate::{build_pb, LOGGER, SAVEOUT_DEPRECATED, SAVEOUT_INCORRECT, SAVEOUT_VALID};
use anyhow::Result;
use image::GenericImageView;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ImageFiles {
    pub map_deprecated_ioerr: HashMap<PathBuf, std::io::Error>,
    pub map_deprecated_imerr: HashMap<PathBuf, image::ImageError>,
    pub map_incorrect_suffix: HashMap<PathBuf, (String, u32, u32)>,
    pub v_valid: Vec<(PathBuf, u32, u32)>,
}
impl ImageFiles {
    pub fn new(paths: &[PathBuf]) -> Result<Self> {
        let pb = build_pb(paths.len() as u64, "Integrity Checking");

        // classify files
        let mut map_deprecated_ioerr: HashMap<PathBuf, std::io::Error> = HashMap::new();
        let mut map_deprecated_imerr: HashMap<PathBuf, image::ImageError> = HashMap::new();
        let mut map_incorrect_suffix: HashMap<PathBuf, (String, u32, u32)> = HashMap::new();
        let mut v_valid: Vec<(PathBuf, u32, u32)> = Vec::new();
        for y in paths.iter() {
            pb.inc(1);
            match image::io::Reader::open(y) {
                Ok(reader) => {
                    let format_given = reader.format();
                    match reader.with_guessed_format() {
                        Ok(reader_guessed) => {
                            let format_guessed = reader_guessed.format();
                            match reader_guessed.decode() {
                                Ok(img) => {
                                    // w, h
                                    let (width, height) = img.dimensions();
                                    // save original path & correct suffix
                                    if format_guessed != format_given {
                                        let src_filestem = y.file_stem().unwrap().to_str().unwrap();
                                        let mime: Vec<&str> = format_guessed
                                            .unwrap()
                                            .to_mime_type()
                                            .split('/')
                                            .collect();
                                        let _suffix = mime.last().unwrap();
                                        let dst = format!("{}.{}", src_filestem, _suffix); // filename supposed
                                        map_incorrect_suffix.insert(
                                            y.canonicalize()?,
                                            (dst.clone(), width, height),
                                        );
                                    } else {
                                        v_valid.push((y.canonicalize()?, width, height));
                                    }
                                }
                                Err(e) => {
                                    map_deprecated_imerr.insert(y.canonicalize()?, e);
                                }
                            }
                        }
                        Err(e) => {
                            map_deprecated_ioerr.insert(y.canonicalize()?, e);
                        }
                    }
                }
                Err(e) => {
                    map_deprecated_ioerr.insert(y.canonicalize()?, e);
                }
            }
        }
        pb.finish();

        // summary
        let cnt_valid = v_valid.len();
        let cnt_deprecated = map_deprecated_imerr.len() + map_deprecated_ioerr.len();
        let cnt_incorrect = map_incorrect_suffix.len();
        let cnt_total = cnt_valid + cnt_deprecated + cnt_incorrect;
        LOGGER.success("Found", &format!("x{}", cnt_total), "");
        LOGGER.success("", SAVEOUT_VALID, &format!("x{}", cnt_valid));
        LOGGER.success("", SAVEOUT_INCORRECT, &format!("x{}", cnt_incorrect));
        LOGGER.success("", SAVEOUT_DEPRECATED, &format!("x{}", cnt_deprecated));
        Ok(ImageFiles {
            v_valid,
            map_incorrect_suffix,
            map_deprecated_imerr,
            map_deprecated_ioerr,
        })
    }

    pub fn has_deprecated(&self) -> bool {
        self.map_deprecated_imerr.len() + self.map_deprecated_ioerr.len() > 0
    }

    pub fn is_ok(&self) -> bool {
        self.map_deprecated_imerr.len()
            + self.map_deprecated_ioerr.len()
            + self.map_incorrect_suffix.len()
            == 0
    }

    pub fn n_total(&self) -> usize {
        self.map_deprecated_imerr.len()
            + self.map_deprecated_ioerr.len()
            + self.map_incorrect_suffix.len()
            + self.v_valid.len()
    }
}
