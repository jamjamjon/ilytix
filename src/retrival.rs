use anyhow::Result;
use std::path::PathBuf;
use usearch::ffi::{IndexOptions, MetricKind, ScalarKind};

use crate::{build_pb, hash2decial, load_files, make_folders, src2dst, Method, LOGGER};

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Kind {
    Image,
    Text,
}

#[derive(clap::Parser, Debug)]
pub struct Retrival {
    #[arg(short, long)]
    input: String,

    #[arg(long)]
    query: String,

    #[arg(short, long, value_enum, default_value_t = Kind::Image)]
    kind: Kind,

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
    verbose: bool,
}

impl Retrival {
    fn build_then_register(&self, paths: &[PathBuf]) -> Result<(usearch::Index, Vec<PathBuf>)> {
        let options = match self.method {
            Method::BlockHash => IndexOptions {
                dimensions: 32,
                metric: MetricKind::Hamming,
                quantization: ScalarKind::F16,
                ..Default::default()
            },
            _ => todo!(),
        };
        let index = usearch::new_index(&options)?;
        index.reserve(paths.len())?;
        let pb = build_pb(paths.len() as u64, "Building");
        let mut v_deprecated: Vec<PathBuf> = Vec::new();
        for (idx, path) in paths.iter().enumerate() {
            pb.inc(1);
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
            match self.method {
                Method::BlockHash => {
                    let hash = blockhash::blockhash256(&img);
                    let hash = hash2decial(&hash.to_string())?;
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
        // load all files & build index & extract feats
        let paths = load_files(&self.input, self.recursive, false, Some("Collection"))?;
        let (index, _) = self.build_then_register(&paths)?;

        let mut v_matched: Vec<usize> = Vec::new();
        match self.kind {
            Kind::Text => todo!(),
            Kind::Image => {
                let img = match image::io::Reader::open(&self.query) {
                    Err(e) => {
                        anyhow::bail!("{:?} => {:?}", e, self.query);
                    }
                    Ok(reader) => match reader.with_guessed_format() {
                        Err(e) => {
                            anyhow::bail!("{:?} => {:?}", e, self.query);
                        }
                        Ok(x) => match x.decode() {
                            Err(e) => {
                                anyhow::bail!("{:?} => {:?}", e, self.query);
                            }
                            Ok(x) => x,
                        },
                    },
                };
                LOGGER.success("Query", &self.query, "");

                match self.method {
                    Method::BlockHash => {
                        let hash = blockhash::blockhash256(&img);
                        let hash = hash2decial(&hash.to_string())?;
                        let matches = index.search(&hash, index.size())?;
                        for (k, score) in
                            matches.keys.into_iter().zip(matches.distances.into_iter())
                        {
                            if score <= self.thresh {
                                v_matched.push(k as usize);
                            }
                        }
                    }
                    _ => todo!(),
                }
            }
        }

        // summary
        LOGGER.success("Matched", &format!("x{}", v_matched.len()), "");
        if v_matched.is_empty() {
            LOGGER.exit("No image retrived", "--thresh", &format!("{}", self.thresh));
        } else if self.verbose {
            for &i in v_matched.iter() {
                LOGGER.success("", paths[i].canonicalize()?.to_str().unwrap(), "");
            }
        }
        match &self.output {
            None => LOGGER.exit(
                "Results",
                "Not Saving",
                "Use `-o <PATH>` to set the save location",
            ),
            Some(output) => {
                let pb = build_pb(
                    v_matched.len() as u64,
                    if !self.mv {
                        "Saving[Copy]"
                    } else {
                        "Saving[Move]"
                    },
                );
                // make dir
                let mut saveout = make_folders(output)?;
                std::fs::create_dir_all(&saveout)?;
                for idx in v_matched {
                    pb.inc(1);
                    let path = &paths[idx];
                    let filename = path.file_name().unwrap().to_str().unwrap();
                    saveout.push(filename);
                    src2dst(path, &saveout, self.mv)?;
                    saveout.pop();
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
}
