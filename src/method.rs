#[derive(Debug, Clone, clap::ValueEnum, Copy)]
pub enum Method {
    BlockHash,
    Nn,
}
