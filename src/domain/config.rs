use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    pub(crate) os: String,
    pub(crate) version: String,
    pub(crate) author: String,
    pub(crate) prompt: Prompt,
    pub(crate) cluster_size: u32,
    pub(crate) cluster_count: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            os: "RoDOS".to_string(),
            version: "0.1.0".to_string(),
            author: "Sami Barbut-Dica".to_string(),
            prompt: Prompt::default(),
            cluster_size: 16,
            cluster_count: 4096,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Prompt {
    pub(crate) host: String,
    pub(crate) separator: String,
    pub(crate) user: String,
    pub(crate) path_prefix: String,
    pub(crate) terminator: String,
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            host: "rodos".to_string(),
            separator: "@".to_string(),
            user: "rouser".to_string(),
            path_prefix: ":".to_string(),
            terminator: "$".to_string(),
        }
    }
}
