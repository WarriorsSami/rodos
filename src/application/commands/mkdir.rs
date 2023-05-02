use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use chrono::{DateTime, Utc};
use mediator::{Request, RequestHandler};

/// MakeDirectoryRequest is a request to make a directory
/// # Fields
/// * `name` - the name of the directory to make
/// * `attributes` - the attributes of the directory to make
/// * `last_modification_datetime` - the last modification datetime of the directory to make
pub(crate) struct MakeDirectoryRequest {
    pub(crate) name: String,
    pub(crate) attributes: u8,
    pub(crate) last_modification_datetime: DateTime<Utc>,
}

impl MakeDirectoryRequest {
    pub(crate) fn new(
        name: String,
        attributes: u8,
        last_modification_datetime: DateTime<Utc>,
    ) -> Self {
        Self {
            name,
            attributes,
            last_modification_datetime,
        }
    }
}

impl Request<Void> for MakeDirectoryRequest {}

/// MakeDirectoryHandler is a handler that handles MakeDirectoryRequests holding a reference to a disk manager
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
