use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use mediator::{Request, RequestHandler};

pub(crate) struct MakeDirectoryRequest {
    pub(crate) name: String,
}

impl MakeDirectoryRequest {
    pub(crate) fn new(name: String) -> Self {
        Self { name }
    }
}

impl Request<Void> for MakeDirectoryRequest {}

pub(crate) struct MakeDirectoryHandler {
    pub(crate) disk_manager: Arm<dyn IDiskManager>,
}

impl MakeDirectoryHandler {
    pub(crate) fn new(disk_manager: Arm<dyn IDiskManager>) -> Self {
        Self { disk_manager }
    }
}

impl RequestHandler<MakeDirectoryRequest, Void> for MakeDirectoryHandler {
    fn handle(&mut self, req: MakeDirectoryRequest) -> Void {
        log::info!("Making directory {}", req.name);

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => {
                disk_manager.pull_sync();

                match disk_manager.make_directory(&req) {
                    Ok(()) => {
                        log::info!("Directory {} has been made successfully", req.name);
                        disk_manager.push_sync();
                        Ok(())
                    }
                    Err(err) => {
                        log::error!("Unable to make directory {}: {}", req.name, err);
                        Err(Box::try_from("Unable to make directory").unwrap())
                    }
                }
            }
            Err(err) => {
                log::error!("Unable to lock disk manager: {}", err);
                Err(Box::try_from("Unable to lock disk manager").unwrap())
            }
        }
    }
}