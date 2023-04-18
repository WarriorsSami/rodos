use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use mediator::{Request, RequestHandler};

pub(crate) struct DefragmentRequest;

impl DefragmentRequest {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Request<Void> for DefragmentRequest {}

pub(crate) struct DefragmentHandler {
    disk_manager: Arm<dyn IDiskManager>,
}

impl DefragmentHandler {
    pub(crate) fn new(disk_manager: Arm<dyn IDiskManager>) -> Self {
        Self { disk_manager }
    }
}

impl RequestHandler<DefragmentRequest, Void> for DefragmentHandler {
    fn handle(&mut self, _req: DefragmentRequest) -> Void {
        log::info!("Defragmenting disk...");

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => match disk_manager.defragment_disk() {
                Ok(()) => {
                    log::info!("Disk has been defragmented successfully");
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Err(_e) => Err(Box::try_from("Unable to lock disk manager!").unwrap()),
        }
    }
}
