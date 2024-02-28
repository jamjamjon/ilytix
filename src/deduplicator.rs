use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;
use usearch::ffi::{IndexOptions, MetricKind, ScalarKind};

use crate::{
    build_pb, load_files, make_folders, LOGGER, SAVEOUT_CURATED, SAVEOUT_DEPRECATED,
    SAVEOUT_DUPLICATED,
};

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

    #[arg(short, long, value_enum, default_value_t = Method::BlockHash)]
    method: Method,

    /// The smaller this parameter is, the lower the tolerance
    #[arg(long, default_value_t = 3.0f32)]
    thresh: f32,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone, clap::ValueEnum, Copy)]
pub enum Method {
    BlockHash,
    Nn,
}

impl DeDuplicator {
    fn hash2decial(&self, s_hash: &str) -> Result<Vec<f32>> {
        let s_hash = s_hash.chars().collect::<Vec<_>>();
        let s = s_hash
            .chunks(2)
            .map(|c| {
                let c = c.iter().collect::<String>();
                u8::from_str_radix(&c, 16).unwrap() as f32
            })
            .collect::<Vec<f32>>();
        Ok(s)
    }

    pub fn encode_hash(&self, path: &PathBuf) -> Result<blockhash::Blockhash256> {
        let img = image::io::Reader::open(path)?
            .with_guessed_format()?
            .decode()?;
        Ok(blockhash::blockhash256(&img))
    }

    fn build_then_register(&self, paths: &[PathBuf]) -> Result<(usearch::Index, Vec<PathBuf>)> {
        let options = match self.method {
            Method::BlockHash => {
                IndexOptions {
                    dimensions: 32,
                    metric: MetricKind::Hamming,
                    quantization: ScalarKind::F16,
                    // connectivity: 0,
                    // expansion_add: 0,
                    // expansion_search: 0,
                    ..Default::default()
                }
            }
            _ => todo!(),
        };
        let index = usearch::new_index(&options)?;
        index.reserve(paths.len())?;
        let pb = build_pb(paths.len() as u64, "Building");
        let mut v_deprecated: Vec<PathBuf> = Vec::new();
        for (idx, path) in paths.iter().enumerate() {
            pb.inc(1);
            // try load image
            let img = match image::io::Reader::open(path) {
                Err(_) => {
                    v_deprecated.push(path.to_path_buf());
                    continue;
                }
                Ok(reader) => match reader.with_guessed_format() {
                    Err(_) => {
                        v_deprecated.push(path.to_path_buf());
                        continue;
                    }
                    Ok(x) => match x.decode() {
                        Err(_) => {
                            v_deprecated.push(path.to_path_buf());
                            continue;
                        }
                        Ok(x) => x,
                    },
                },
            };

            // register
            match self.method {
                Method::BlockHash => {
                    let hash = blockhash::blockhash256(&img);
                    let hash = self.hash2decial(&hash.to_string())?;
                    index.add(idx as u64, &hash)?;
                }
                _ => todo!(),
            }
        }
        pb.finish();

        // index
        LOGGER.success("Index", "", "");
        LOGGER.success("", "Capacity", &format!("{}", index.capacity()));
        LOGGER.success("", "Size", &format!("{}", index.size()));
        LOGGER.success("", "Dimensions", &format!("{}", index.dimensions()));
        if index.size() <= 1 {
            LOGGER.exit(
                "Error",
                "Too few images to deduplicate",
                &format!("{}", index.size()),
            );
        }

        Ok((index, v_deprecated))
    }

    pub fn run(&self) -> Result<()> {
        // load all files, make sure that all paths is valid image
        let mut paths = load_files(&self.input, self.recursive, false)?;

        // sort by file size
        paths.sort_by(|a, b| {
            let size_a = std::fs::metadata(a).unwrap().len();
            let size_b = std::fs::metadata(b).unwrap().len();
            size_b.partial_cmp(&size_a).unwrap()
        });

        // build index & extract all images feats
        let (index, v_deprecated) = self.build_then_register(&paths)?;

        let mut set_duplicates: HashSet<usize> = HashSet::new();
        let pb = build_pb(paths.len() as u64, "Deduplicating");

        // feat buffer
        let mut feats: [f32; 32] = [0.0f32; 32];
        for (idx, _path) in paths.iter().enumerate() {
            pb.inc(1);

            // non-image
            if !index.contains(idx as u64) {
                continue;
            }

            // duplicated
            if set_duplicates.contains(&idx) {
                continue;
            }

            // filter
            index.get(idx as u64, &mut feats)?;
            let matches = index.search(&feats, index.size())?;
            for (&key, &dist) in matches.keys.iter().zip(matches.distances.iter()) {
                if key == idx as u64 {
                    continue;
                }
                if dist <= self.thresh {
                    set_duplicates.insert(key as usize);
                }
            }
        }
        pb.finish();

        // summary
        LOGGER.success("Found", "", "");
        LOGGER.success(
            "",
            SAVEOUT_DUPLICATED,
            &format!("x{}", set_duplicates.len()),
        );
        LOGGER.success(
            "",
            SAVEOUT_CURATED,
            &format!("x{}", index.size() - set_duplicates.len()),
        );
        LOGGER.success("", SAVEOUT_DEPRECATED, &format!("x{}", v_deprecated.len()));

        // verbose, deprecated
        if self.verbose && !v_deprecated.is_empty() {
            LOGGER.warn("Unsupported Or Deprecated Files", "", "");
            for p in v_deprecated.iter() {
                LOGGER.warn("", &format!("{}", p.canonicalize()?.display()), "");
            }
        }

        if set_duplicates.is_empty() {
            println!(
                "\n🎉 All the images seem non-duplicate under the current threshold: {}, but you can still deduplicate by adjusting the threshold.",
                self.thresh
            );
            if !v_deprecated.is_empty() {
                println!(
                    "⚠️ But, Still, there are {:?} files that are unsupported or deprecated.",
                    v_deprecated.len()
                );
            }
            return Ok(());
        }

        match &self.output {
            None => LOGGER.exit(
                "Results save at",
                "None",
                "Use `-o <PATH>` to set the save location",
            ),
            Some(output) => {
                // build pb
                let pb = build_pb(
                    paths.len() as u64,
                    if !self.mv {
                        "Saving(Copy)"
                    } else {
                        "Saving(Move)"
                    },
                );

                // build dir
                let saveout = make_folders(output)?;
                let mut saveout_duplicated = saveout.clone();
                let mut saveout_curated = saveout.clone();
                saveout_duplicated.push(SAVEOUT_DUPLICATED);
                saveout_curated.push(SAVEOUT_CURATED);
                std::fs::create_dir_all(&saveout_duplicated)?;
                std::fs::create_dir_all(&saveout_curated)?;

                for (idx, path) in paths.iter().enumerate() {
                    pb.inc(1);
                    let filename = path.file_name().unwrap().to_str().unwrap();

                    // save
                    if set_duplicates.contains(&idx) {
                        saveout_duplicated.push(filename);
                        self.save(path, &saveout_duplicated, self.mv)?;
                        saveout_duplicated.pop();
                    } else if v_deprecated.contains(path) {
                        continue;
                    } else {
                        saveout_curated.push(filename);
                        self.save(path, &saveout_curated, self.mv)?;
                        saveout_curated.pop();
                    }
                }

                pb.finish();

                // summary
                LOGGER.success(
                    "Results saved at",
                    &format!("{}", saveout.canonicalize()?.display()),
                    "",
                );
            }
        }

        Ok(())
    }

    fn save(&self, src: &PathBuf, dst: &PathBuf, mv: bool) -> Result<()> {
        if mv {
            match std::fs::rename(src, dst) {
                Ok(_) => {}
                Err(e) => LOGGER.exit(
                    "Error when moving",
                    &format!("{}", e),
                    &format!("{}", dst.canonicalize()?.display()),
                ),
            }
        } else {
            match std::fs::copy(src, dst) {
                Ok(_) => {}
                Err(e) => LOGGER.exit(
                    "Error when copying",
                    &format!("{}", e),
                    &format!("{}", dst.canonicalize()?.display()),
                ),
            }
        }
        Ok(())
    }
}
