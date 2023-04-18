use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use mediator::{Request, RequestHandler};

pub(crate) struct FormatRequest {
    pub(crate) fat_type: u16,
}

impl FormatRequest {
    pub(crate) fn new(fat_type: u16) -> Self {
        Self { fat_type }
    }
}

impl Request<Void> for FormatRequest {}

pub(crate) struct FormatHandler {
    disk_manager: Arm<dyn IDiskManager>,
}

impl FormatHandler {
    pub(crate) fn new(disk_manager: Arm<dyn IDiskManager>) -> Self {
        Self { disk_manager }
    }
}

impl RequestHandler<FormatRequest, Void> for FormatHandler {
    fn handle(&mut self, req: FormatRequest) -> Void {
        log::info!("Formatting disk with FAT type {}", req.fat_type);

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => match disk_manager.format_disk(&req) {
                Ok(()) => {
                    log::info!("Disk has been formatted successfully");
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Err(_e) => Err(Box::try_from("Unable to lock disk manager!").unwrap()),
        }
    }
}
