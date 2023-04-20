use crate::application::Void;
use crate::core::config::Config;
use crate::core::Arm;
use crate::{info, success, warn};
use color_print::cprintln;
use mediator::{Request, RequestHandler};

pub(crate) struct HelpRequest {
    pub(crate) command: Option<String>,
}

impl HelpRequest {
    pub(crate) fn new(command: Option<String>) -> Self {
        Self { command }
    }
}

impl Request<Void> for HelpRequest {}

pub(crate) struct HelpHandler {
    config: Arm<Config>,
}

impl HelpHandler {
    pub(crate) fn new(config: Arm<Config>) -> Self {
        Self { config }
    }
}

impl RequestHandler<HelpRequest, Void> for HelpHandler {
    fn handle(&mut self, request: HelpRequest) -> Void {
        log::info!("Showing help...");

        match self.config.lock() {
            Ok(config) => match request.command {
                Some(command) => match config.commands.get(&command) {
                    Some(command) => {
                        let command_name = command.name.clone();
                        success!("{}:", command_name);

                        let command_description = command.description.clone();
                        info!("{}", command_description);

                        let command_usage = command.usage.clone();
                        warn!("{}", command_usage);

                        log::info!("Help shown successfully!");
                        Ok(())
                    }
                    None => Err(Box::try_from("Command not found").unwrap()),
                },
                None => {
                    info!("Available commands:");
                    config.commands.iter().for_each(|(_name, command)| {
                        cprintln!("<g!>{}</> - {}", command.name, command.description)
                    });

                    log::info!("Help shown successfully!");
                    Ok(())
                }
            },
            Err(_) => Err(Box::try_from("Unable to lock config").unwrap()),
        }
    }
}
