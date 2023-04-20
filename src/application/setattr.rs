use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use mediator::{Request, RequestHandler};

pub(crate) struct SetAttributesRequest {
    pub(crate) name: String,
    pub(crate) extension: String,
    pub(crate) attributes: u8,
}

impl SetAttributesRequest {
    pub(crate) fn new(name: String, extension: String, attributes: u8) -> Self {
        Self {
            name,
            extension,
            attributes,
        }
    }
}

impl Request<Void> for SetAttributesRequest {}

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
            "Setting attributes {} for file {}",
            request.attributes,
            request.name
        );

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => match disk_manager.set_attributes(&request) {
                Ok(_) => {
                    log::info!("Attributes set successfully");
                    disk_manager.push_sync();
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Err(_) => Err(Box::try_from("Unable to lock disk manager").unwrap()),
        }
    }
}
