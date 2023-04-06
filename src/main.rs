use crate::application::neofetch::{NeofetchHandler, NeofetchRequest};
use crate::domain::config::Config;
use crate::domain::disk::Disk;
use color_print::*;
use lazy_static::lazy_static;
use mediator::{DefaultMediator, Mediator};
use std::io::Write;
use std::sync::{Arc, Mutex};

mod application;
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
    pub(crate) static ref CONFIG_ARC: Arc<Mutex<Config>> = Arc::new(Mutex::new(CONFIG.clone()));
    pub(crate) static ref DISK: Disk = {
        let mut disk = Disk::new(CONFIG.clone());
        disk.sync_to_buffer();
        disk.sync_to_file();
        disk
    };
    pub(crate) static ref DISK_ARC: Arc<Mutex<Disk>> = Arc::new(Mutex::new(DISK.clone()));
    pub(crate) static ref MEDIATOR: DefaultMediator = DefaultMediator::builder()
        .add_handler(NeofetchHandler::new(CONFIG_ARC.clone()))
        .build();
}

fn main() {
    let mut mediator = MEDIATOR.clone();

    loop {
        cprint!(
            "<w!>{}</><b!>{}</><w!>{}</><b!>{}</>/<b!>{}</> ",
            CONFIG.prompt.host,
            CONFIG.prompt.separator,
            CONFIG.prompt.user,
            CONFIG.prompt.path_prefix,
            CONFIG.prompt.terminator
        );
        // Flush the buffer to print the prompt before reading the input
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let command = input.split_whitespace().next().unwrap();

        match command {
            "neofetch" => {
                mediator.send(NeofetchRequest).unwrap();
            }
            "exit" => {
                cprintln!("<r!>RoDOS is shutting down!</>");
                break;
            }
            _ => cprintln!("<r>Command not found!</>"),
        }
    }
}
