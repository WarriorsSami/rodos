use crate::application::neofetch::{NeofetchHandler, NeofetchRequest};
use crate::domain::config::Config;
use color_print::*;
use mediator::{DefaultMediator, Mediator};
use std::io::Write;
use std::sync::{Arc, Mutex};

mod application;
mod domain;

fn main() {
    // config
    let config_str = std::fs::read_to_string("config.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_str).expect("Unable to parse config string");

    let config_arc: Arc<Mutex<Config>> = Arc::new(Mutex::new(
        toml::from_str(&config_str).expect("Unable to parse config string"),
    ));
    let mut mediator = DefaultMediator::builder()
        .add_handler(NeofetchHandler::new(config_arc))
        .build();

    // main loop
    loop {
        cprint!(
            "<w!>{}</><b!>{}</><w!>{}</><g!>{}</> ",
            config.prompt.host,
            config.prompt.separator,
            config.prompt.user,
            config.prompt.terminator
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
