use crate::application::Void;
use crate::core::filter_type::FilterType;
use crate::core::sort_type::SortType;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use color_print::cprintln;
use mediator::{Request, RequestHandler};

pub(crate) struct ListRequest {
    pub(crate) filters: Vec<FilterType>,
    pub(crate) sort: Option<SortType>,
}

impl ListRequest {
    pub(crate) fn new(filters: Vec<FilterType>, sort: Option<SortType>) -> Self {
        Self { filters, sort }
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
    fn handle(&mut self, request: ListRequest) -> Void {
        log::info!("Listing files...");

        match self.disk_manager.lock() {
            Ok(mut disk_manager) => {
                disk_manager.pull_sync();

                match disk_manager.list_files(&request) {
                    Ok(file_entries) => {
                        cprintln!(
                            "<w!>Current dir `{}`</>: <b!>{} file(s)</>",
                            disk_manager.get_working_directory(),
                            file_entries.len()
                        );

                        if request.filters.contains(&FilterType::InShortFormat) {
                            file_entries.iter().for_each(|file_entry| {
                                println!("{}.{}", file_entry.name, file_entry.extension);
                            });
                        } else {
                            file_entries.iter().for_each(|file_entry| {
                                println!("{}", file_entry);
                            });
                        }

                        println!();
                        cprintln!("<g!>Free space:</> {} B", disk_manager.get_free_space());
                        cprintln!("<g!>Total space:</> {} B", disk_manager.get_total_space());

                        log::info!("Listed files successfully");
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            Err(_) => Err(Box::try_from("Unable to lock disk manager!").unwrap()),
        }
    }
}
