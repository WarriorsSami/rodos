use crate::application::Void;
use crate::core::Arm;
use crate::domain::file_entry::FileEntryAttributes;
use crate::domain::i_disk_manager::IDiskManager;
use mediator::{Request, RequestHandler};

/// SetAttributesRequest is a request to set attributes for a file
/// # Fields
/// * `name` - the name of the file
/// * `extension` - the extension of the file (empty if directory)
/// * `attributes` - the attributes to set
pub(crate) struct SetAttributesRequest {
    pub(crate) name: String,
    pub(crate) extension: String,
    pub(crate) attributes: Vec<FileEntryAttributes>,
}

impl SetAttributesRequest {
    pub(crate) fn new(
        name: String,
        extension: String,
        attributes: Vec<FileEntryAttributes>,
    ) -> Self {
        Self {
            name,
            extension,
            attributes,
        }
    }
}

impl Request<Void> for SetAttributesRequest {}

/// SetAttributesHandler is a handler for SetAttributesRequest holding a reference to the disk manager
pub(crate) struct SetAttributesHandler {
    disk_manager: Arm<dyn IDiskManager>,
}

impl SetAttributesHandler {
    pub(crate) fn new(disk_manager: Arm<dyn IDiskManager>) -> Self {
        Self { disk_manager }
    }
}

impl RequestHandler<SetAttributesRequest, Void> for SetAttributesHandler {
    fn handle(&mut self, request: SetAttributesRequest) -> Void {
        log::info!(
            "Setting attributes {:?} for file {}",
            request.attributes,
            request.name
        );

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => {
                disk_manager.pull_sync();

                match disk_manager.set_attributes(&request) {
                    Ok(_) => {
                        log::info!("Attributes set successfully");
                        disk_manager.push_sync();
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            Err(_) => Err(Box::try_from("Unable to lock disk manager").unwrap()),
        }
    }
}
