use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use color_print::cprintln;
use mediator::{Request, RequestHandler};

pub(crate) struct CatRequest {
    pub(crate) file_name: String,
    pub(crate) file_extension: String,
}

impl CatRequest {
    pub(crate) fn new(file_name: String, file_extension: String) -> Self {
        Self {
            file_name,
            file_extension,
        }
    }
}

impl Request<Void> for CatRequest {}

pub(crate) struct CatHandler {
    disk_manager: Arm<dyn IDiskManager>,
}

impl CatHandler {
    pub(crate) fn new(disk_manager: Arm<dyn IDiskManager>) -> Self {
        Self { disk_manager }
    }
}

impl RequestHandler<CatRequest, Void> for CatHandler {
    fn handle(&mut self, request: CatRequest) -> Void {
        log::info!(
            "Showing file {}.{}",
            request.file_name,
            request.file_extension
        );

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => match disk_manager.get_file_content(&request) {
                Ok(content) => {
                    cprintln!(
                        "File <b!>{}.{}</> content is:\n<g!>{}</>",
                        request.file_name,
                        request.file_extension,
                        content
                    );

                    log::info!(
                        "Content for file {}.{} has been shown successfully",
                        request.file_name,
                        request.file_extension,
                    );
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Err(_e) => Err(Box::try_from("Unable to lock disk manager!").unwrap()),
        }
    }
}
