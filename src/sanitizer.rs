use anyhow::Result;

use crate::{
    build_pb, load_files, make_folders, src2dst, ImageFiles, LOGGER, SAVEOUT_DEPRECATED,
    SAVEOUT_FILTERED, SAVEOUT_INCORRECT, SAVEOUT_RECTIFIED, SAVEOUT_VALID,
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

    #[arg(long, default_value_t = 0)]
    min_width: u32,

    #[arg(long, default_value_t = 0)]
    min_height: u32,
}

impl Sanitizer {
    pub fn run(&self) -> Result<()> {
        let paths = load_files(&self.input, self.recursive, false, None)?;
        let files = ImageFiles::new(&paths)?;
        if files.is_ok() {
            println!("\n🎉 All the images appear to be intact and accurate.");
            return Ok(());
        }

        // save
        match &self.output {
            None => LOGGER.exit(
                "Results",
                "Not Saving",
                "Use `-o <PATH>` to set the save location",
            ),
            Some(output) => {
                let pb = build_pb(
                    files.n_total() as u64,
                    if !self.mv {
                        "Saving[Copy]"
                    } else {
                        "Saving[Move]"
                    },
                );
                let saveout = make_folders(output)?;

                // build saveout_filtered
                let mut saveout_filtered = saveout.clone();
                saveout_filtered.push(SAVEOUT_FILTERED);
                std::fs::create_dir_all(&saveout_filtered)?;
                let mut n_filtered = 0;

                // deal with valid
                if !files.v_valid.is_empty() {
                    let mut saveout = saveout.clone();
                    saveout.push(SAVEOUT_VALID);
                    std::fs::create_dir_all(&saveout)?;

                    for (f, w, h) in files.v_valid.iter() {
                        pb.inc(1);
                        if w >= &self.min_width && h >= &self.min_height {
                            saveout.push(f.file_name().unwrap());
                            src2dst(f, &saveout, self.mv)?;
                            saveout.pop();
                        } else {
                            saveout_filtered.push(f.file_name().unwrap());
                            src2dst(f, &saveout_filtered, self.mv)?;
                            saveout_filtered.pop();
                            n_filtered += 1;
                        }
                    }
                }

                // deal with deprecated
                if !files.map_deprecated_imerr.is_empty() && files.map_deprecated_ioerr.is_empty() {
                    let mut saveout = saveout.clone();
                    saveout.push(SAVEOUT_DEPRECATED);
                    std::fs::create_dir_all(&saveout)?;
                    for f in files
                        .map_deprecated_imerr
                        .keys()
                        .chain(files.map_deprecated_ioerr.keys())
                    {
                        pb.inc(1);
                        saveout.push(f.file_name().unwrap());
                        src2dst(f, &saveout, self.mv)?;
                        saveout.pop();
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

                    for (f, (filename, w, h)) in files.map_incorrect_suffix.iter() {
                        pb.inc(1);
                        // save incorrect
                        saveout_incorrect.push(f.file_name().unwrap());
                        src2dst(f, &saveout_incorrect, self.mv)?;
                        saveout_incorrect.pop();

                        // save rectified
                        if w >= &self.min_width && h >= &self.min_height {
                            saveout_rectified.push(filename);
                            src2dst(f, &saveout_rectified, self.mv)?;
                            saveout_rectified.pop();
                        } else {
                            saveout_filtered.push(filename);
                            src2dst(f, &saveout_filtered, self.mv)?;
                            saveout_filtered.pop();
                            n_filtered += 1;
                        }
                    }
                }
                pb.finish();

                // cleanup
                if n_filtered == 0 {
                    std::fs::remove_dir_all(saveout_filtered)?;
                }
                if self.min_width != 0 || self.min_height != 0 {
                    LOGGER.success("Images filtered out", &format!("x{}", n_filtered), "");
                    if self.min_width != 0 {
                        LOGGER.success("", "Min width", &format!("{}", self.min_width));
                    }
                    if self.min_width != 0 {
                        LOGGER.success("", "Min height", &format!("{}", self.min_height));
                    }
                }
                // saveout
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
