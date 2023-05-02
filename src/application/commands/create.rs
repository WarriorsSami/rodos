use crate::application::Void;
use crate::core::content_type::ContentType;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use chrono::{DateTime, Utc};
use color_print::cprintln;
use mediator::{Request, RequestHandler};

/// CreateRequest is a request to create a file
/// # Fields
/// * `name` - the name of the file
/// * `extension` - the extension of the file
/// * `size` - the size of the file
/// * `attributes` - the attributes of the file
/// * `last_modification_datetime` - the last modification datetime of the file
/// * `content_type` - the content type of the file
pub(crate) struct CreateRequest {
    pub(crate) name: String,
    pub(crate) extension: String,
    pub(crate) size: u32,
    pub(crate) attributes: u8,
    pub(crate) last_modification_datetime: DateTime<Utc>,
    pub(crate) content_type: ContentType,
}

impl CreateRequest {
    pub(crate) fn new(
        name: String,
        extension: String,
        size: u32,
        attributes: u8,
        last_modification_datetime: DateTime<Utc>,
        content_type: ContentType,
    ) -> Self {
        Self {
            name,
            extension,
            size,
            attributes,
            last_modification_datetime,
            content_type,
        }
    }
}

impl Request<Void> for CreateRequest {}

pub(crate) struct CreateHandler {
    disk_manager: Arm<dyn IDiskManager>,
}

impl CreateHandler {
    pub(crate) fn new(disk_manager: Arm<dyn IDiskManager>) -> Self {
        Self { disk_manager }
    }
}

impl RequestHandler<CreateRequest, Void> for CreateHandler {
    fn handle(&mut self, request: CreateRequest) -> Void {
        log::info!(
            "Creating file {}.{} with dimension {} and content type {}",
            request.name,
            request.extension,
            request.size,
            request.content_type
        );
        cprintln!(
            "Creating file <b!>{}.{}</> with dimension <y!>{}</> and content type <y!>{}</>...",
            request.name,
            request.extension,
            request.size,
            request.content_type
        );

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => {
                disk_manager.pull_sync();

                match disk_manager.create_file(&request) {
                    Ok(_) => {
                        log::info!("Created file successfully");
                        disk_manager.push_sync();
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            Err(_) => Err(Box::try_from("Unable to lock disk manager!").unwrap()),
        }
    }
}
