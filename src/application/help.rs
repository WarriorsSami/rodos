use crate::application::Void;
use crate::core::config::Config;
use crate::core::Arm;
use color_print::cprintln;
use mediator::{Request, RequestHandler};

pub(crate) struct HelpRequest {}

impl HelpRequest {
    pub(crate) fn new() -> Self {
        Self {}
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
    fn handle(&mut self, _request: HelpRequest) -> Void {
        log::info!("Showing help...");

        match self.config.lock() {
            Ok(config) => {
                cprintln!("<c!>Available commands:</>");
                config.commands.iter().for_each(|(_name, command)| {
                    cprintln!("<g!>{}</> - {}", command.name, command.description)
                });

                log::info!("Help shown successfully!");
                Ok(())
            }
            Err(_) => Err(Box::try_from("Unable to lock config").unwrap()),
        }
    }
}
