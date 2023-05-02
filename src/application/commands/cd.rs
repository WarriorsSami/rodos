use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use color_print::cprintln;
use mediator::{Request, RequestHandler};

/// Request to change the working directory
/// # Fields
/// * `directory_name` - The name of the directory to change to
pub(crate) struct ChangeDirectoryRequest {
    pub(crate) directory_name: String,
}

impl ChangeDirectoryRequest {
    pub(crate) fn new(directory_name: String) -> Self {
        Self { directory_name }
    }
}

impl Request<Void> for ChangeDirectoryRequest {}

/// Handler for ChangeDirectoryRequest holding a reference to the disk manager
pub(crate) struct ChangeDirectoryHandler {
    disk_manager: Arm<dyn IDiskManager>,
}

impl ChangeDirectoryHandler {
    pub(crate) fn new(disk_manager: Arm<dyn IDiskManager>) -> Self {
        Self { disk_manager }
    }
}

impl RequestHandler<ChangeDirectoryRequest, Void> for ChangeDirectoryHandler {
    fn handle(&mut self, request: ChangeDirectoryRequest) -> Void {
        log::info!("Changing directory to {}...", request.directory_name);

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => {
                disk_manager.pull_sync();

                match disk_manager.change_working_directory(&request) {
                    Ok(_) => {
                        log::info!(
                            "Changed directory successfully to {}",
                            disk_manager.get_working_directory_full_path()
                        );
                        cprintln!(
                            "<g!>Changed directory successfully to</> <c!>{}</>",
                            disk_manager.get_working_directory_full_path()
                        );
                        Ok(())
                    }
                    Err(err) => {
                        log::error!("Failed to change directory: {}", err);
                        Err(err)
                    }
                }
            }
            Err(err) => {
                log::error!("Failed to change directory: {}", err);
                Err(Box::try_from("Failed to change directory").unwrap())
            }
        }
    }
}
