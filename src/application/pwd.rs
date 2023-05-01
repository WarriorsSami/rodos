use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use crate::info;
use color_print::cprintln;
use mediator::{Request, RequestHandler};

pub(crate) struct PwdRequest;

impl PwdRequest {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Request<Void> for PwdRequest {}

pub(crate) struct PwdHandler {
    disk_manager: Arm<dyn IDiskManager>,
}

impl PwdHandler {
    pub(crate) fn new(disk_manager: Arm<dyn IDiskManager>) -> Self {
        Self { disk_manager }
    }
}

impl RequestHandler<PwdRequest, Void> for PwdHandler {
    fn handle(&mut self, _req: PwdRequest) -> Void {
        log::info!("Showing current directory");

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => {
                disk_manager.pull_sync();
                let current_directory = disk_manager.get_working_directory_full_path();
                info!("{}", current_directory);

                log::info!("Current directory is {}", current_directory);
                Ok(())
            }
            Err(_) => Err(Box::try_from("Failed to lock disk manager").unwrap()),
        }
    }
}
