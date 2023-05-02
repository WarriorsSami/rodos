use crate::application::commands::cd::ChangeDirectoryHandler;
use crate::application::commands::cp::CopyHandler;
use crate::application::commands::create::CreateHandler;
use crate::application::commands::defrag::DefragmentHandler;
use crate::application::commands::del::DeleteHandler;
use crate::application::commands::fmt::FormatHandler;
use crate::application::commands::mkdir::MakeDirectoryHandler;
use crate::application::commands::rename::RenameHandler;
use crate::application::commands::setattr::SetAttributesHandler;
use crate::application::queries::cat::CatHandler;
use crate::application::queries::help::HelpHandler;
use crate::application::queries::ls::ListHandler;
use crate::application::queries::neofetch::NeofetchHandler;
use crate::application::queries::pwd::PwdHandler;
use crate::core::cli_parser::CliParser;
use crate::core::config::Config;
use crate::core::Arm;
use crate::domain::boot_sector::BootSector;
use crate::domain::i_disk_manager::IDiskManager;
use crate::infrastructure::disk_manager::DiskManager;
use color_print::{cprint, cprintln};
use lazy_static::lazy_static;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use mediator::{DefaultMediator, Mediator};
use std::io::Write;
use std::sync::{Arc, Mutex};

mod application;
mod core;
mod domain;
mod infrastructure;

// config
lazy_static! {
    /// Config is a singleton that holds the configuration for the entire application
    pub(crate) static ref CONFIG: Config = {
        let config_res = std::fs::read_to_string("config/config.toml");

        let config: Config = match config_res {
            Ok(config_str) => toml::from_str(&config_str).expect("Unable to parse config string"),
            Err(..) => Config::default(),
        };

        // create disk folder if it doesn't exist
        if !std::path::Path::new(&config.disk_dir_path).exists() {
            std::fs::create_dir(&config.disk_dir_path).expect("Unable to create disk folder");
        }

        // create stdin and temp files if they don't exist
        // stdin support is not currently implemented
        if !std::path::Path::new(&config.stdin_file_path).exists() {
            std::fs::File::create(&config.stdin_file_path).expect("Unable to create stdin file");
        }

        if !std::path::Path::new(&config.temp_file_path).exists() {
            std::fs::File::create(&config.temp_file_path).expect("Unable to create temp file");
        }

        config
    };
    pub(crate) static ref CONFIG_ARC: Arm<Config> = Arc::new(Mutex::new(CONFIG.clone()));
    /// Disk manager singleton wrapped in an Arc<Mutex<>> to allow for concurrent access (not currently used)
    pub(crate) static ref DISK_ARC: Arm<dyn IDiskManager> = {
        let mut disk_manager = DiskManager::new(CONFIG_ARC.clone(), BootSector::default());

        // if storage file doesn't exist or is empty, create new storage file based on in-memory config
        // otherwise, init disk manager from storage file
        let storage_file_exists = std::path::Path::new(&CONFIG.storage_file_path).exists();
        if !storage_file_exists {
            disk_manager.push_sync();
        } else {
            let storage_file_size = std::fs::metadata(&CONFIG.storage_file_path)
                .expect("Unable to get storage file metadata")
                .len();

            if storage_file_size == 0 {
                disk_manager.push_sync();
            } else {
                disk_manager.pull_boot_sector_sync(); // grab the boot sector from the storage file

                // create new disk manager according to the boot sector from the storage file
                // this is necessary in order to tackle the inconsistencies between the in-memory
                // data structures used to represent the disk when switching between FAT16 and FAT32 and vice-versa
                disk_manager = DiskManager::new(CONFIG_ARC.clone(), disk_manager.get_boot_sector().clone());

                disk_manager.pull_sync(); // grab the rest of the data from the storage file
            }
        }

        Arc::new(Mutex::new(disk_manager))
    };
    /// The mediator is responsible for redirecting commands to the appropriate handlers
    pub(crate) static ref MEDIATOR: DefaultMediator = DefaultMediator::builder()
        .add_handler(HelpHandler::new(CONFIG_ARC.clone()))
        .add_handler(NeofetchHandler::new(CONFIG_ARC.clone(), DISK_ARC.clone()))
        .add_handler(CreateHandler::new(DISK_ARC.clone()))
        .add_handler(ListHandler::new(DISK_ARC.clone()))
        .add_handler(RenameHandler::new(DISK_ARC.clone()))
        .add_handler(DeleteHandler::new(DISK_ARC.clone()))
        .add_handler(CatHandler::new(DISK_ARC.clone()))
        .add_handler(CopyHandler::new(DISK_ARC.clone()))
        .add_handler(FormatHandler::new(DISK_ARC.clone()))
        .add_handler(DefragmentHandler::new(DISK_ARC.clone()))
        .add_handler(SetAttributesHandler::new(DISK_ARC.clone()))
        .add_handler(MakeDirectoryHandler::new(DISK_ARC.clone()))
        .add_handler(ChangeDirectoryHandler::new(DISK_ARC.clone()))
        .add_handler(PwdHandler::new(DISK_ARC.clone()))
        .build();
}

fn main() {
    init_logger();
    let mut mediator = MEDIATOR.clone();

    log::info!("RoDOS is booting up...");
    loop {
        prompt!();

        // read input from stdin
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(read_bytes) => {
                log::info!("Read {} bytes from stdin", read_bytes);
            }
            Err(err) => {
                warn!("Unable to read input, please try again!");

                log::warn!("Unable to read input, please try again! Error: {}", err);
                continue;
            }
        }
        // get the first word of the input - this is the command
        let command = input.split_whitespace().next();

        if command.is_none() {
            warn!("Please enter a command!");
            continue;
        }

        // match the command to the appropriate handler
        match command.unwrap() {
            "neofetch" => handle!(mediator, parse_neofetch, input.as_str()),
            "create" => handle!(
                mediator,
                parse_create,
                input.as_str(),
                "File created successfully!"
            ),
            "ls" => handle!(mediator, parse_ls, input.as_str()),
            "rename" => handle!(
                mediator,
                parse_rename,
                input.as_str(),
                "File renamed successfully!"
            ),
            "del" => handle!(
                mediator,
                parse_del,
                input.as_str(),
                "File deleted successfully!"
            ),
            "cat" => handle!(mediator, parse_cat, input.as_str()),
            "cp" => handle!(
                mediator,
                parse_cp,
                input.as_str(),
                "File copied successfully!"
            ),
            "setattr" => handle!(
                mediator,
                parse_setattr,
                input.as_str(),
                "File attributes set successfully!"
            ),
            "fmt" => handle!(
                mediator,
                parse_fmt,
                input.as_str(),
                "Disk formatted successfully",
                reboot_system,
                "The system requires a reboot in order to properly persist the modifications!\nRoDOS is shutting down..."
            ),
            "defrag" => handle!(
                mediator,
                parse_defrag,
                input.as_str(),
                "Disk defragmented successfully"
            ),
            "mkdir" => handle!(
                mediator,
                parse_mkdir,
                input.as_str(),
                "Directory created successfully!"
            ),
            "cd" => handle!(mediator, parse_cd, input.as_str()),
            "pwd" => handle!(mediator, parse_pwd, input.as_str()),
            "rmdir" => handle!(
                mediator,
                parse_rmdir,
                input.as_str(),
                "Directory deleted successfully!"
            ),
            "help" => handle!(mediator, parse_help, input.as_str()),
            "exit" => handle!(
                parse_exit,
                input.as_str(),
                reboot_system,
                "RoDOS is shutting down..."
            ),
            _ => {
                warn!("Warning: Command not found!");
            }
        }
    }
}

fn init_logger() {
    match log4rs::init_file("config/log4rs.yaml", Default::default()) {
        Ok(_) => {}
        Err(_) => {
            let file_appender = FileAppender::builder()
                .encoder(Box::new(PatternEncoder::new(
                    "{d(%Y-%m-%d %H:%M:%S%.3f)} {h({l})} {M} - {m}{n}",
                )))
                .build("logs/rodos.logs")
                .unwrap();

            let log_config = log4rs::config::Config::builder()
                .appender(Appender::builder().build("file_appender", Box::new(file_appender)))
                .logger(
                    Logger::builder()
                        .appender("file_appender")
                        .additive(false)
                        .build("rodos", log::LevelFilter::Info),
                )
                .build(
                    Root::builder()
                        .appender("file_appender")
                        .build(log::LevelFilter::Info),
                )
                .unwrap();

            log4rs::init_config(log_config).unwrap();
        }
    }
}

fn reboot_system(bye_message: &str) {
    warn!("Warning: {}", bye_message);

    log::info!("RoDOS is shutting down...");
    std::process::exit(0);
}
