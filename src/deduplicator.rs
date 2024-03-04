use crate::{
    build_pb, load_files, make_folders, src2dst, Method, LOGGER, SAVEOUT_CURATED,
    SAVEOUT_DEPRECATED, SAVEOUT_DUPLICATED,
};
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
pub struct DeDuplicator {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    recursive: bool,

    #[arg(long)]
    mv: bool,

    #[arg(long, value_enum, default_value_t = Method::BlockHash)]
    method: Method,

    /// The smaller this parameter is, the lower the tolerance
    #[arg(long, default_value_t = 3.0f32)]
    thresh: f32,

    #[arg(short, long)]
    show_deprecated: bool,
}

impl DeDuplicator {
    pub fn run(&self) -> Result<()> {
        let paths = load_files(&self.input, self.recursive, false, None)?;
        let pb = build_pb(paths.len() as u64, "Deduplicating");
        let mut maps_curated: HashMap<PathBuf, blockhash::Blockhash256> = HashMap::new();
        let mut v_dups: Vec<PathBuf> = Vec::new();
        let mut v_deps: Vec<PathBuf> = Vec::new();
        for path in &paths {
            pb.inc(1);
            // try load
            let img = match image::io::Reader::open(path) {
                Err(_) => {
                    v_deps.push(path.to_path_buf());
                    continue;
                }
                Ok(reader) => match reader.with_guessed_format() {
                    Err(_) => {
                        v_deps.push(path.to_path_buf());
                        continue;
                    }
                    Ok(x) => match x.decode() {
                        Err(_) => {
                            v_deps.push(path.to_path_buf());
                            continue;
                        }
                        Ok(x) => x,
                    },
                },
            };
            let feat = blockhash::blockhash256(&img);
            let mut _v_dup: Vec<(u64, PathBuf, blockhash::Blockhash256)> = Vec::new();
            maps_curated.iter().for_each(|(p, f)| {
                if feat.distance(f) <= self.thresh as u32 {
                    _v_dup.push((
                        std::fs::metadata(p).unwrap().len(),
                        p.to_path_buf(),
                        f.to_owned(),
                    ));
                }
            });

            // deal with duplicates
            if _v_dup.is_empty() {
                maps_curated.insert(path.to_path_buf(), feat);
            } else {
                _v_dup.push((
                    std::fs::metadata(path).unwrap().len(),
                    path.to_path_buf(),
                    feat.to_owned(),
                ));

                // choose the best and remove the others
                _v_dup.par_sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
                for (idx, (_size, path, feat)) in _v_dup.iter().enumerate() {
                    if idx == 0 {
                        maps_curated.insert(path.to_path_buf(), *feat);
                        continue;
                    }
                    maps_curated.remove(path);
                    v_dups.push(path.to_path_buf());
                }
            }
        }
        pb.finish();

        // summary
        LOGGER.success("Found", "", "");
        LOGGER.success("", SAVEOUT_DUPLICATED, &format!("x{}", v_dups.len()));
        LOGGER.success("", SAVEOUT_CURATED, &format!("x{}", maps_curated.len()));
        LOGGER.success("", SAVEOUT_DEPRECATED, &format!("x{}", v_deps.len()));

        // show deprecated
        if self.show_deprecated {
            if v_deps.is_empty() {
                LOGGER.success("Unsupported Files Or Deprecated Images", "Not Found", "");
            } else {
                LOGGER.warn("Unsupported Files Or Deprecated Images", "", "");
                for p in v_deps.iter() {
                    LOGGER.warn("", &format!("{}", p.canonicalize()?.display()), "");
                }
            }
        }

        if v_dups.is_empty() {
            println!(
                "\n🎉 All the images seem non-duplicate under the current threshold: {}",
                self.thresh
            );
            if !v_deps.is_empty() {
                println!(
                    "\n❗️ Note that there are {:?} files that are unsupported or deprecated",
                    v_deps.len()
                );
            }
            return Ok(());
        }

        match &self.output {
            None => LOGGER.exit(
                "Results",
                "Not Saving",
                "Use `-o <PATH>` to set the save location",
            ),
            Some(output) => {
                let pb = build_pb(
                    (maps_curated.len() + v_dups.len()) as u64,
                    if !self.mv {
                        "Saving[Copy]"
                    } else {
                        "Saving[Move]"
                    },
                );
                // make dirs
                let saveout = make_folders(output)?;
                let mut saveout_dups = saveout.clone();
                let mut saveout_curated = saveout.clone();
                saveout_dups.push(SAVEOUT_DUPLICATED);
                saveout_curated.push(SAVEOUT_CURATED);
                std::fs::create_dir_all(&saveout_dups)?;
                std::fs::create_dir_all(&saveout_curated)?;
                for path in maps_curated.into_keys() {
                    pb.inc(1);
                    let name = path.file_name().unwrap().to_str().unwrap();
                    saveout_curated.push(name);
                    src2dst(&path, &saveout_curated, self.mv)?;
                    saveout_curated.pop();
                }
                for path in v_dups.into_iter() {
                    pb.inc(1);
                    let name = path.file_name().unwrap().to_str().unwrap();
                    saveout_dups.push(name);
                    src2dst(&path, &saveout_dups, self.mv)?;
                    saveout_dups.pop();
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
