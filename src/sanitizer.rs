use anyhow::Result;

use crate::{
    build_pb, load_files, make_folders, ImageFiles, LOGGER, SAVEOUT_DEPRECATED, SAVEOUT_INCORRECT,
    SAVEOUT_RECTIFIED, SAVEOUT_VALID,
};

#[derive(clap::Parser, Debug)]
pub struct Sanitizer {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    recursive: bool,

    #[arg(short, long)]
    mv: bool,
    // #[arg(short, long)]
    // verbose: bool,
}

impl Sanitizer {
    pub fn run(&self) -> Result<()> {
        // load all files
        let paths = load_files(&self.input, self.recursive, false)?;
        let files = ImageFiles::new(&paths)?;

        // early return
        if files.is_ok() {
            println!("\n🎉 All the images appear to be intact and accurate.");
            return Ok(());
        }

        // save
        match &self.output {
            None => LOGGER.exit(
                "Results save at",
                "None",
                "Use `-o <PATH>` to set the save location",
            ),
            Some(output) => {
                let pb = build_pb(
                    files.n_total() as u64,
                    if !self.mv {
                        "Saving(Copy)"
                    } else {
                        "Saving(Move)"
                    },
                );
                let saveout = make_folders(output)?;

                // deal with valid
                if !files.v_valid.is_empty() {
                    let mut saveout = saveout.clone();
                    saveout.push(SAVEOUT_VALID);
                    std::fs::create_dir_all(&saveout)?;
                    for f in files.v_valid.iter() {
                        pb.inc(1);
                        let filename = f.file_name().unwrap().to_str().unwrap();
                        let dst =
                            format!("{}/{}", saveout.canonicalize()?.to_str().unwrap(), filename);
                        if self.mv {
                            std::fs::rename(f, dst)?;
                        } else {
                            std::fs::copy(f, dst)?;
                        }
                    }
                }

                // deal with deprecated
                if !files.map_deprecated_imerr.is_empty() && files.map_deprecated_ioerr.is_empty() {
                    let mut saveout = saveout.clone();
                    saveout.push(SAVEOUT_DEPRECATED);
                    std::fs::create_dir_all(&saveout)?;
                    for f in files.map_deprecated_imerr.keys() {
                        pb.inc(1);
                        let filename = f.file_name().unwrap().to_str().unwrap();
                        let dst =
                            format!("{}/{}", saveout.canonicalize()?.to_str().unwrap(), filename);
                        if self.mv {
                            std::fs::rename(f, dst)?;
                        } else {
                            std::fs::copy(f, dst)?;
                        }
                    }
                    for f in files.map_deprecated_ioerr.keys() {
                        pb.inc(1);
                        let filename = f.file_name().unwrap().to_str().unwrap();
                        let dst =
                            format!("{}/{}", saveout.canonicalize()?.to_str().unwrap(), filename);
                        if self.mv {
                            std::fs::rename(f, dst)?;
                        } else {
                            std::fs::copy(f, dst)?;
                        }
                    }
                }

                // deal with incorrect
                if !files.map_incorrect_suffix.is_empty() {
                    let mut saveout_incorrect = saveout.clone();
                    let mut saveout_rectified = saveout.clone();
                    saveout_incorrect.push(SAVEOUT_INCORRECT);
                    saveout_rectified.push(SAVEOUT_RECTIFIED);
                    std::fs::create_dir_all(&saveout_incorrect)?;
                    std::fs::create_dir_all(&saveout_rectified)?;
                    for (f, filename) in files.map_incorrect_suffix.iter() {
                        pb.inc(1);
                        let img = image::io::Reader::open(f)?
                            .with_guessed_format()?
                            .decode()?;
                        let dst = format!(
                            "{}/{}",
                            saveout_rectified.canonicalize()?.to_str().unwrap(),
                            filename
                        );
                        match img.save(dst) {
                            Ok(_) => {
                                let filename = f.file_name().unwrap().to_str().unwrap();
                                let dst = format!(
                                    "{}/{}",
                                    saveout_incorrect.canonicalize()?.to_str().unwrap(),
                                    filename
                                );
                                if self.mv {
                                    std::fs::rename(f, dst)?;
                                } else {
                                    std::fs::copy(f, dst)?;
                                }
                            }
                            Err(e) => LOGGER.warn(
                                "Failed to save",
                                &format!("{}", e),
                                &format!("{}", f.display()),
                            ),
                        }
                    }
                }
                pb.finish();
                LOGGER.success(
                    "Results saved at",
                    &format!("{}", saveout.canonicalize()?.display()),
                    "",
                );
            }
        }

        Ok(())
    }
}
