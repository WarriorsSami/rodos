use crate::application::create::CreateRequest;
use crate::application::Void;
use crate::domain::file_entry::RootTable;
use std::error::Error;

pub(crate) trait IDiskManager: Sync + Send {
    /// Synchronizes the disk manager with the storage file.
    /// This method should be called after every command request, i.e. every operation that modifies the storage.
    /// ## Errors
    /// * `Box<dyn Error>` - If the disk manager is not able to sync with the storage file.
    fn push_sync(&mut self);

    /// Synchronizes the storage file with the disk manager.
    /// This method should be called before every query request, i.e. every operation that only inquires the storage.
    /// ## Errors
    /// * `Box<dyn Error>` - If the disk manager is not able to sync with the storage file.
    fn pull_sync(&mut self);

    /// Creates a file with the given parameters.
    /// Returns an error if the file already exists or there is not enough space in the disk.
    /// Otherwise, returns `Ok(())`.
    /// ## Arguments
    /// * `request` - The request containing the file parameters.
    /// ## Errors
    /// * `Box<dyn Error>` - If the file already exists or there is not enough space in the disk.
    fn create_file(&mut self, request: CreateRequest) -> Void;

    /// List all the files from the working directory
    /// ## Errors
    /// * `Box<dyn Error>` - If the disk manager is not able to sync with the storage file.
    fn list_files(&mut self) -> Result<RootTable, Box<dyn Error>>;

    /// Returns the working directory
    fn get_working_directory(&self) -> String;
}