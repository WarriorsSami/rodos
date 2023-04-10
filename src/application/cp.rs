use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use mediator::{Request, RequestHandler};

pub(crate) struct CopyRequest {
    pub(crate) src_name: String,
    pub(crate) src_extension: String,
    pub(crate) dest_name: String,
    pub(crate) dest_extension: String,
}

impl CopyRequest {
    pub(crate) fn new(
        src_name: String,
        src_extension: String,
        dest_name: String,
        dest_extension: String,
    ) -> Self {
        Self {
            src_name,
            src_extension,
            dest_name,
            dest_extension,
        }
    }
}

impl Request<Void> for CopyRequest {}

pub(crate) struct CopyHandler {
    disk_manager: Arm<dyn IDiskManager>,
}

impl CopyHandler {
    pub(crate) fn new(disk_manager: Arm<dyn IDiskManager>) -> Self {
        Self { disk_manager }
    }
}

impl RequestHandler<CopyRequest, Void> for CopyHandler {
    fn handle(&mut self, request: CopyRequest) -> Void {
        log::info!("Copying file...");

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => match disk_manager.copy_file(&request) {
                Ok(()) => {
                    log::info!(
                        "Copied file successfully from {}.{} to {}.{}",
                        request.src_name,
                        request.src_extension,
                        request.dest_name,
                        request.dest_extension
                    );
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Err(_) => Err(Box::try_from("Unable to lock disk manager!").unwrap()),
        }
    }
}
