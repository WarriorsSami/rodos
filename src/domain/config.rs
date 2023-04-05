use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) os: String,
    pub(crate) version: String,
    pub(crate) author: String,
    pub(crate) prompt: Prompt,
    pub(crate) cluster_size: u32,
    pub(crate) cluster_count: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Prompt {
    pub(crate) host: String,
    pub(crate) separator: String,
    pub(crate) user: String,
    pub(crate) terminator: String,
}
