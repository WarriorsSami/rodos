use crate::application::neofetch::{NeofetchHandler, NeofetchRequest};
use color_print::*;
use mediator::{DefaultMediator, Mediator};
use std::io::Write;

mod application;
mod domain;

fn main() {
    let mut mediator = DefaultMediator::builder()
        .add_handler(NeofetchHandler::new())
        .build();

    loop {
        cprint!("<w!>rouser</><b!>@</><w!>rodos</>-rs:~$ ");
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
            _ => println!("Command not found"),
        }
    }
}
