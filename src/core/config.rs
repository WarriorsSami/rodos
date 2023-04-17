use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    pub(crate) os: String,
    pub(crate) version: String,
    pub(crate) author: String,
    pub(crate) prompt: Prompt,
    pub(crate) commands: Commands,
    pub(crate) storage_file_path: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut commands = HashMap::new();

        commands.insert(
            "help".to_string(),
            Command {
                name: "help".to_string(),
                description: "Display the list of available commands".to_string(),
                usage: "help".to_string(),
                regex: r"^\s*help\s*$".to_string(),
            },
        );

        commands.insert(
            "exit".to_string(),
            Command {
                name: "exit".to_string(),
                description: "Exit the shell".to_string(),
                usage: "exit".to_string(),
                regex: r"^\s*exit\s*$".to_string(),
            },
        );

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

        commands.insert(
            "del".to_string(),
            Command {
                name: "del".to_string(),
                description: "Delete a file or a directory".to_string(),
                usage: "del <file_name>.<file_extension>".to_string(),
                regex: r"^\s*del\s+(?P<name>\S+)\.(?P<extension>\S+)\s*$".to_string(),
            },
        );

        commands.insert(
            "cat".to_string(),
            Command {
                name: "cat".to_string(),
                description: "Display the content of a file".to_string(),
                usage: "cat <file_name>.<file_extension>".to_string(),
                regex: r"^\s*cat\s+(?P<name>\S+)\.(?P<extension>\S+)\s*$".to_string(),
            },
        );

        commands.insert(
            "cp".to_string(),
            Command {
                name: "cp".to_string(),
                description: "Copy a file".to_string(),
                usage: "cp <file_name>.<file_extension> <new_file_name>.<new_file_extension>".to_string(),
                regex: r"^\s*cp\s+(?P<src_name>\S+)\.(?P<src_extension>\S+)\s+(?P<dest_name>\S+)\.(?P<dest_extension>\S+)\s*$".to_string(),
            },
        );

        Self {
            os: "RoDOS".to_string(),
            version: "0.1.0".to_string(),
            author: "Sami Barbut-Dica".to_string(),
            prompt: Prompt::default(),
            commands,
            storage_file_path: "disk/storage.bin".to_string(),
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
