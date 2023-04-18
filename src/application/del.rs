use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use color_print::cprintln;
use mediator::{Request, RequestHandler};

pub(crate) struct DeleteRequest {
    pub(crate) file_name: String,
    pub(crate) file_extension: String,
}

impl DeleteRequest {
    pub(crate) fn new(file_name: String, file_extension: String) -> Self {
        Self {
            file_name,
            file_extension,
        }
    }
}

impl Request<Void> for DeleteRequest {}

pub(crate) struct DeleteHandler {
    disk_manager: Arm<dyn IDiskManager>,
}

impl DeleteHandler {
    pub(crate) fn new(disk_manager: Arm<dyn IDiskManager>) -> Self {
        Self { disk_manager }
    }
}

impl RequestHandler<DeleteRequest, Void> for DeleteHandler {
    fn handle(&mut self, request: DeleteRequest) -> Void {
        log::info!(
            "Deleting file {}.{}...",
            request.file_name,
            request.file_extension
        );
        cprintln!(
            "Deleting file <b!>{}.{}</>...",
            request.file_name,
            request.file_extension
        );

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => match disk_manager.delete_file(&request) {
                Ok(_) => {
                    log::info!("Deleted file successfully");
                    disk_manager.push_sync();
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Err(_) => Err(Box::try_from("Unable to lock disk manager!").unwrap()),
        }
    }
}
