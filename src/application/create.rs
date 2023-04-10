use crate::application::Void;
use crate::core::content_type::ContentType;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use color_print::cprintln;
use mediator::{Request, RequestHandler};

pub(crate) struct CreateRequest {
    pub(crate) file_name: String,
    pub(crate) file_extension: String,
    pub(crate) file_size: u32,
    pub(crate) content_type: ContentType,
}

impl CreateRequest {
    pub(crate) fn new(
        file_name: String,
        file_extension: String,
        file_size: u32,
        content_type: ContentType,
    ) -> Self {
        Self {
            file_name,
            file_extension,
            file_size,
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
            request.file_name,
            request.file_extension,
            request.file_size,
            request.content_type
        );
        cprintln!(
            "Creating file <b!>{}.{}</> with dimension <y!>{}</> and content type <y!>{}</>...",
            request.file_name,
            request.file_extension,
            request.file_size,
            request.content_type
        );

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => match disk_manager.create_file(request) {
                Ok(_) => {
                    log::info!("Created file successfully");
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Err(_) => Err(Box::try_from("Unable to lock disk manager!").unwrap()),
        }
    }
}
