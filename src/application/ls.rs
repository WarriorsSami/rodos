use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use color_print::cprintln;
use mediator::{Request, RequestHandler};

pub(crate) struct ListRequest;

impl ListRequest {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Request<Void> for ListRequest {}

pub(crate) struct ListHandler {
    disk_manager: Arm<dyn IDiskManager>,
}

impl ListHandler {
    pub(crate) fn new(disk_manager: Arm<dyn IDiskManager>) -> Self {
        Self { disk_manager }
    }
}

impl RequestHandler<ListRequest, Void> for ListHandler {
    fn handle(&mut self, _request: ListRequest) -> Void {
        log::info!("Listing files...");

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => match disk_manager.list_files() {
                Ok(file_entries) => {
                    cprintln!(
                        "<w!>Current dir `{}`</>: <b!>{} file(s)</>",
                        disk_manager.get_working_directory(),
                        file_entries.len()
                    );

                    file_entries.iter().for_each(|file_entry| {
                        println!("{}", file_entry);
                    });

                    println!();
                    cprintln!("<g!>Free space:</> {} B", disk_manager.get_free_space());
                    cprintln!("<g!>Total space:</> {} B", disk_manager.get_total_space());

                    log::info!("Listed files successfully");
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Err(_) => Err(Box::try_from("Unable to lock disk manager!").unwrap()),
        }
    }
}
