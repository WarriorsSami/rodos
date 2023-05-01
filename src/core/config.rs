use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    pub(crate) os: String,
    pub(crate) version: String,
    pub(crate) author: String,
    pub(crate) prompt: Prompt,
    pub(crate) commands: Commands,
    pub(crate) disk_dir_path: String,
    pub(crate) storage_file_path: String,
    pub(crate) stdin_file_path: String,
    pub(crate) temp_file_path: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut commands = HashMap::new();

        commands.insert(
            "help".to_string(),
            Command {
                name: "help".to_string(),
                description: "Display the list of available commands".to_string(),
                usage: "help [command]".to_string(),
                regex: r"^\s*help(\s+(?P<command>\S+))?\s*$".to_string(),
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
                usage: "create <file_name>.<file_extension> <file_size> -<file_content_type>".to_string(),
                regex: r"^\s*create\s+(?P<name>\S+)\.(?P<extension>\S+)\s+(?P<dim>\d+)\s+-(?P<type>\S+)\s*$".to_string(),
            },
        );

        commands.insert(
            "ls".to_string(),
            Command {
                name: "ls".to_string(),
                description: "List files and directories from the current directory by applying given filters and sorting options".to_string(),
                usage: "ls [-<filter>] [-name=<file_name>] [-ext=<file_extension>] [-<sort>]\n-<filter>:\n\t-a: show all visible files and directories\n\t-h: show all files and directories including hidden ones\n\t-s: show files and directories in short format (name and extension)\n\t-l: show files and directories in detailed format (attributes, name, extension, last modification date and size in bytes)\n\t-f: show all files\n\t-d: show all directories\n-<sort>:\n\t-n: sort by name\n\t-t: sort by last modification date\n\t-sz: sort by size\n\t-*a: sort in ascending order\n\t-*d: sort in descending order".to_string(),
                regex: r"^\s*ls(\s+-(?P<filter_basic>(a|h)(s|l)?(f|d)?))?(\s+-name=(?P<filter_name>\S+))?(\s+-ext=(?P<filter_extension>\S+))?(\s+-(?P<sort>(n|t|sz)(a|d)))?\s*$".to_string(),
            },
        );

        commands.insert(
            "rename".to_string(),
            Command {
                name: "rename".to_string(),
                description: "Rename a file or a directory".to_string(),
                usage: "rename <old_name> <new_name>".to_string(),
                regex: r"^\s*rename\s+(?P<old_name>[a-zA-Z0-9_]+)(\.(?P<old_extension>\S+))?\s+(?P<new_name>[a-zA-Z0-9_]+)(\.(?P<new_extension>\S+))?\s*$".to_string(),
            },
        );

        commands.insert(
            "del".to_string(),
            Command {
                name: "del".to_string(),
                description: "Delete a file or a directory".to_string(),
                usage: "del <file_name>.<file_extension> or del <directory_name>".to_string(),
                regex: r"^\s*del\s+(?P<name>[a-zA-Z0-9_]+)(\.(?P<extension>\S+))?\s*$".to_string(),
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
                regex: r"^\s*cp\s+(?P<src_name>[a-zA-Z0-9_]+)(\.(?P<src_extension>\S+))?\s+(?P<dest_name>[a-zA-Z0-9_]+)(\.(?P<dest_extension>\S+))?\s*$".to_string(),
            },
        );

        commands.insert(
            "fmt".to_string(),
            Command {
                name: "fmt".to_string(),
                description:
                    "Format the disk using the specified FAT cluster size and reboot the system"
                        .to_string(),
                usage: "fmt 16/32".to_string(),
                regex: r"^\s*fmt\s+(?P<fat_type>(16|32))\s*$".to_string(),
            },
        );

        commands.insert(
            "defrag".to_string(),
            Command {
                name: "defrag".to_string(),
                description: "Defragment the disk".to_string(),
                usage: "defrag".to_string(),
                regex: r"^\s*defrag\s*$".to_string(),
            },
        );

        commands.insert(
            "setattr".to_string(),
            Command {
                name: "setattr".to_string(),
                description: "Set the attributes of a file or a directory".to_string(),
                usage: "setattr <file_name>.<file_extension> <attributes>(max 2 blocks, e.g. +w-h, but not +w-h+h)\n<attributes>:\n\t+w: make read-write\n\t-w: make read-only\n\t+h: make hidden\n\t-h: make visible".to_string(),
                regex: r"^\s*setattr\s+(?P<name>[a-zA-Z0-9_]+)(\.(?P<extension>\S+))?\s+(?P<attributes>((\+|-)(w|h)){1,2})\s*$".to_string(),
            }
        );

        commands.insert(
            "mkdir".to_string(),
            Command {
                name: "mkdir".to_string(),
                description: "Create a new directory".to_string(),
                usage: "mkdir <directory_name>".to_string(),
                regex: r"^\s*mkdir\s+(?P<name>\S+)\s*$".to_string(),
            },
        );

        commands.insert(
            "cd".to_string(),
            Command {
                name: "cd".to_string(),
                description: "Change the current directory".to_string(),
                usage: "cd <directory_name>".to_string(),
                regex: r"^\s*cd\s+(?P<name>\S+)\s*$".to_string(),
            },
        );

        commands.insert(
            "pwd".to_string(),
            Command {
                name: "pwd".to_string(),
                description: "Display the current directory".to_string(),
                usage: "pwd".to_string(),
                regex: r"^\s*pwd\s*$".to_string(),
            },
        );

        Self {
            os: "RoDOS".to_string(),
            version: "0.1.0".to_string(),
            author: "Sami Barbut-Dica".to_string(),
            prompt: Prompt::default(),
            commands,
            disk_dir_path: "disk".to_string(),
            storage_file_path: "disk/storage.bin".to_string(),
            stdin_file_path: "disk/stdin.in".to_string(),
            temp_file_path: "disk/temp".to_string(),
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
