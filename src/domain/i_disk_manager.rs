use crate::application::cat::CatRequest;
use crate::application::create::CreateRequest;
use crate::application::del::DeleteRequest;
use crate::application::rename::RenameRequest;
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
    /// * `Box<dyn Error>` - If the file already exists or there is not enough space in the disk or
    /// a file with the same name already exists.
    fn create_file(&mut self, request: &CreateRequest) -> Void;

    /// List all the files from the working directory
    /// ## Errors
    /// * `Box<dyn Error>` - If the disk manager is not able to sync with the storage file.
    fn list_files(&mut self) -> Result<RootTable, Box<dyn Error>>;

    /// Renames a file with the given name.
    /// ## Arguments
    /// * `request` - The request containing the old and the new names.
    /// ## Errors
    /// * `Box<dyn Error>` - If the old file does not exist or a file with the same name as the new one already exists.
    fn rename_file(&mut self, request: &RenameRequest) -> Void;

    /// Deletes a file with the given name.
    /// ## Arguments
    /// * `request` - The request containing the file name and the file extension.
    /// ## Errors
    /// * `Box<dyn Error>` - If the file does not exist.
    fn delete_file(&mut self, request: &DeleteRequest) -> Void;

    /// Displays the content of a file with the given name.
    /// ## Arguments
    /// * `request` - The request containing the file name and the file extension.
    /// ## Errors
    /// * `Box<dyn Error>` - If the file does not exist.
    fn get_file_content(&mut self, request: &CatRequest) -> Result<String, Box<dyn Error>>;

    /// Returns the working directory
    fn get_working_directory(&self) -> String;

    /// Returns the free space in the disk with respect to the total number of empty clusters
    fn get_free_space(&mut self) -> u64;

    /// Returns the total space in the disk
    fn get_total_space(&self) -> u64;
}
