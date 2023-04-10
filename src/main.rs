use crate::application::cat::CatHandler;
use crate::application::create::CreateHandler;
use crate::application::del::DeleteHandler;
use crate::application::help::HelpHandler;
use crate::application::ls::ListHandler;
use crate::application::neofetch::NeofetchHandler;
use crate::application::rename::RenameHandler;
use crate::core::cli_parser::CliParser;
use crate::core::config::Config;
use crate::core::Arm;
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
    pub(crate) static ref CONFIG: Config = {
        let config_res = std::fs::read_to_string("config/config.toml");

        let config: Config = match config_res {
            Ok(config_str) => toml::from_str(&config_str).expect("Unable to parse config string"),
            Err(..) => Config::default(),
        };

        config
    };
    pub(crate) static ref CONFIG_ARC: Arm<Config> = Arc::new(Mutex::new(CONFIG.clone()));
    pub(crate) static ref DISK_ARC: Arm<dyn IDiskManager> = {
        let mut disk_manager = DiskManager::new(CONFIG_ARC.clone());

        // if storage file doesn't exist, is empty or tampered, create new storage file based on in-memory config
        // otherwise, init disk manager from storage file
        let storage_file_exists = std::path::Path::new(&CONFIG.storage_file_path).exists();
        if !storage_file_exists {
            disk_manager.push_sync();
        } else {
            let storage_file_size = std::fs::metadata(&CONFIG.storage_file_path)
                .expect("Unable to get storage file metadata")
                .len();

            if storage_file_size != disk_manager.get_total_space() {
                disk_manager.push_sync();
            } else {
                disk_manager.pull_sync();
            }
        }

        Arc::new(Mutex::new(disk_manager))
    };
    pub(crate) static ref MEDIATOR: DefaultMediator = DefaultMediator::builder()
        .add_handler(HelpHandler::new(CONFIG_ARC.clone()))
        .add_handler(NeofetchHandler::new(CONFIG_ARC.clone()))
        .add_handler(CreateHandler::new(DISK_ARC.clone()))
        .add_handler(ListHandler::new(DISK_ARC.clone()))
        .add_handler(RenameHandler::new(DISK_ARC.clone()))
        .add_handler(DeleteHandler::new(DISK_ARC.clone()))
        .add_handler(CatHandler::new(DISK_ARC.clone()))
        .build();
}

fn main() {
    init_logger();
    let mut mediator = MEDIATOR.clone();

    log::info!("RoDOS is booting up...");
    loop {
        prompt!();

        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(..) => {}
            Err(err) => {
                warn!("Unable to read input, please try again!");

                log::warn!("Unable to read input, please try again! Error: {}", err);
                continue;
            }
        }
        let command = input.split_whitespace().next();

        if command.is_none() {
            warn!("Please enter a command!");
            continue;
        }

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
            "help" => handle!(mediator, parse_help, input.as_str()),
            "exit" => match CliParser::parse_exit(input.as_str()) {
                Ok(_) => {
                    warn!("RoDOS is shutting down...");

                    log::info!("RoDOS is booting down...");
                    std::process::exit(0);
                }
                Err(err) => {
                    warn!(err);
                    log::warn!("{}", err);
                }
            },
            _ => {
                warn!("Command not found!");
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
