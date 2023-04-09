use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    pub(crate) os: String,
    pub(crate) version: String,
    pub(crate) author: String,
    pub(crate) prompt: Prompt,
    pub(crate) commands: Commands,
    pub(crate) cluster_size: u32,
    pub(crate) cluster_count: u32,
    pub(crate) storage_file_path: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut commands = HashMap::new();

        commands.insert(
            "neofetch".to_string(),
            Command {
                name: "neofetch".to_string(),
                description: "Display system information".to_string(),
                usage: "neofetch".to_string(),
                regex: r"^\s*neofetch\s*$".to_string(),
            },
        );

        commands.insert(
            "create".to_string(),
            Command {
                name: "create".to_string(),
                description: "Create a new file".to_string(),
                usage: "create <file_name>.<file_extension> <file_size> <file_content_type>".to_string(),
                regex: r"^\s*create\s+(?P<name>\S+)\.(?P<extension>\S+)\s+(?P<dim>\d+)\s+(?P<type>\S+)\s*$".to_string(),
            },
        );

        commands.insert(
            "ls".to_string(),
            Command {
                name: "ls".to_string(),
                description: "List files and directories from the current directory".to_string(),
                usage: "ls".to_string(),
                regex: r"^\s*ls\s*$".to_string(),
            },
        );

        commands.insert(
            "rename".to_string(),
            Command {
                name: "rename".to_string(),
                description: "Rename a file or a directory".to_string(),
                usage: "rename <old_name> <new_name>".to_string(),
                regex: r"^\s*rename\s+(?P<old_name>\S+)\.(?P<old_extension>\S+)\s+(?P<new_name>\S+)\.(?P<new_extension>\S+)\s*$".to_string(),
            },
        );

        Self {
            os: "RoDOS".to_string(),
            version: "0.1.0".to_string(),
            author: "Sami Barbut-Dica".to_string(),
            prompt: Prompt::default(),
            commands,
            cluster_size: 16,
            cluster_count: 4096,
            storage_file_path: "storage.bin".to_string(),
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

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Command {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) usage: String,
    pub(crate) regex: String,
}

pub(crate) type Commands = HashMap<String, Command>;
