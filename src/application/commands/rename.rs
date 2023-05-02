use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use mediator::{Request, RequestHandler};

/// RenameRequest is a request to rename a file
/// # Fields
/// * `old_name` - the old name of the file
/// * `old_extension` - the old extension of the file (empty if directory)
/// * `new_name` - the new name of the file
/// * `new_extension` - the new extension of the file (empty if directory)
#[derive(Debug, Clone)]
pub(crate) struct RenameRequest {
    pub(crate) old_name: String,
    pub(crate) old_extension: String,
    pub(crate) new_name: String,
    pub(crate) new_extension: String,
}

impl RenameRequest {
    pub(crate) fn new(
        old_name: String,
        old_extension: String,
        new_name: String,
        new_extension: String,
    ) -> Self {
        Self {
            old_name,
            old_extension,
            new_name,
            new_extension,
        }
    }
}

impl Request<Void> for RenameRequest {}

/// RenameHandler is a handler for RenameRequest holding a reference to the disk manager
pub(crate) struct RenameHandler {
    disk_manager: Arm<dyn IDiskManager>,
}

impl RenameHandler {
    pub(crate) fn new(disk_manager: Arm<dyn IDiskManager>) -> Self {
        Self { disk_manager }
    }
}

impl RequestHandler<RenameRequest, Void> for RenameHandler {
    fn handle(&mut self, request: RenameRequest) -> Void {
        log::info!("Renaming file...");

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => match disk_manager.rename_file(&request) {
                Ok(_) => {
                    log::info!(
                        "File {}.{} renamed successfully to {}.{}!",
                        request.old_name,
                        request.old_extension,
                        request.new_name,
                        request.new_extension
                    );
                    disk_manager.push_sync();
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Err(_) => Err(Box::try_from("Unable to lock disk manager!").unwrap()),
        }
    }
}
