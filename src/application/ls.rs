use crate::application::Void;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use mediator::{Request, RequestHandler};

pub(crate) struct ListRequest;

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
        match self.disk_manager.lock() {
            Ok(mut disk_manager) => match disk_manager.list_files() {
                Ok(file_entries) => {
                    println!(
                        "Current directory: {}",
                        disk_manager.get_working_directory()
                    );

                    file_entries.iter().for_each(|file_entry| {
                        println!("{}", file_entry);
                    });

                    println!();
                    println!("Free space: {} B", disk_manager.get_free_space());
                    println!("Total space: {} B", disk_manager.get_total_space());

                    Ok(())
                }
                Err(e) => Err(e),
            },
            Err(_) => Err(Box::try_from("Unable to lock disk manager!").unwrap()),
        }
    }
}
