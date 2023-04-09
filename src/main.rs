use crate::application::create::CreateHandler;
use crate::application::ls::{ListHandler, ListRequest};
use crate::application::neofetch::{NeofetchHandler, NeofetchRequest};
use crate::core::cli_parser::CliParser;
use crate::core::Arm;
use crate::domain::config::Config;
use crate::domain::disk_manager::DiskManager;
use crate::domain::i_disk_manager::IDiskManager;
use color_print::*;
use lazy_static::lazy_static;
use mediator::{DefaultMediator, Mediator};
use std::io::Write;
use std::sync::{Arc, Mutex};

mod application;
mod core;
mod domain;

// config
lazy_static! {
    pub(crate) static ref CONFIG: Config = {
        let config_res = std::fs::read_to_string("config.toml");

        let config: Config = match config_res {
            Ok(config_str) => toml::from_str(&config_str).expect("Unable to parse config string"),
            Err(..) => Config::default(),
        };

        config
    };
    pub(crate) static ref CONFIG_ARC: Arm<Config> = Arc::new(Mutex::new(CONFIG.clone()));
    pub(crate) static ref DISK_ARC: Arm<dyn IDiskManager> = {
        let mut disk_manager = DiskManager::new(CONFIG_ARC.clone());
        disk_manager.push_sync();
        Arc::new(Mutex::new(disk_manager))
    };
    pub(crate) static ref MEDIATOR: DefaultMediator = DefaultMediator::builder()
        .add_handler(NeofetchHandler::new(CONFIG_ARC.clone()))
        .add_handler(CreateHandler::new(DISK_ARC.clone()))
        .add_handler(ListHandler::new(DISK_ARC.clone()))
        .build();
}

fn main() {
    let mut mediator = MEDIATOR.clone();

    loop {
        prompt!();

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Unable to read input");
        let command = input.split_whitespace().next();

        if command.is_none() {
            warn!("Please enter a command!");
            continue;
        }

        match command.unwrap() {
            "neofetch" => {
                if let Err(err) = mediator.send(NeofetchRequest) {
                    error!(err);
                }
            }
            "create" => match CliParser::parse_create(input.as_str()) {
                Ok(request) => {
                    if let Err(err) = mediator.send(request).unwrap() {
                        error!(err);
                    } else {
                        success!("File created successfully!");
                    }
                }
                Err(err) => {
                    warn!(err);
                }
            },
            "ls" => {
                if let Err(err) = mediator.send(ListRequest).unwrap() {
                    warn!(err);
                }
            }
            "exit" => {
                warn!("RoDOS is shutting down!");
                break;
            }
            _ => {
                warn!("Command not found!");
            }
        }
    }
}
