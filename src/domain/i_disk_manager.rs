use crate::application::cat::CatRequest;
use crate::application::cd::ChangeDirectoryRequest;
use crate::application::cp::CopyRequest;
use crate::application::create::CreateRequest;
use crate::application::del::DeleteRequest;
use crate::application::fmt::FormatRequest;
use crate::application::ls::ListRequest;
use crate::application::mkdir::MakeDirectoryRequest;
use crate::application::rename::RenameRequest;
use crate::application::setattr::SetAttributesRequest;
use crate::application::Void;
use crate::domain::boot_sector::BootSector;
use crate::domain::file_entry::RootTable;
use std::error::Error;

pub(crate) trait IDiskManager: Sync + Send {
    /// Propagates the latest changes from the disk manager to the storage file.
    /// This method should be called after every command request, i.e. every operation that modifies the storage.
    /// ## Errors
    /// * `Box<dyn Error>` - If the disk manager is not able to sync with the storage file.
    fn push_sync(&mut self);

    /// Brings into the in-memory disk manager the latest changes from the storage file.
    /// This method should be called before every query request, i.e. every operation that only inquires the storage.
    /// ## Errors
    /// * `Box<dyn Error>` - If the disk manager is not able to sync with the storage file.
    fn pull_sync(&mut self);

    /// Brings into the in-memory disk manager the latest changes from the storage file regarding the boot sector.
    /// This method is specially designed to be used only when initializing the disk manager after a format operation.
    /// ### Errors
    /// * `Box<dyn Error>` - If the disk manager is not able to sync with the storage file.
    fn pull_boot_sector_sync(&mut self);

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
    /// ## Arguments
    /// * `request` - The request containing the filters and the sort type.
    /// ## Errors
    /// * `Box<dyn Error>` - If the disk manager is not able to sync with the storage file.
    fn list_files(&mut self, request: &ListRequest) -> Result<RootTable, Box<dyn Error>>;

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

    /// Copies a file with the given name.
    /// ## Arguments
    /// * `request` - The request containing the file name and the file extension for the source file and the destination file.
    /// ## Errors
    /// * `Box<dyn Error>` - If the source file does not exist or a file with the same name as the destination file already exists.
    fn copy_file(&mut self, request: &CopyRequest) -> Void;

    /// Set attributes for a given file or directory.
    /// ## Arguments
    /// * `request` - The request containing the file/directory name and the attributes to set.
    /// ## Errors
    /// * `Box<dyn Error>` - If the file/directory does not exist.
    fn set_attributes(&mut self, request: &SetAttributesRequest) -> Void;

    /// Formats the disk
    /// ## Arguments
    /// * `request` - The request containing the FAT type.
    /// ## Errors
    /// * `Box<dyn Error>` - If the disk is not able to be formatted.
    fn format_disk(&mut self, request: &FormatRequest) -> Void;

    /// Defragments the disk
    /// ## Errors
    /// * `Box<dyn Error>` - If the disk is not able to be defragmented.
    fn defragment_disk(&mut self) -> Void;

    /// Creates a new directory in the working directory.
    /// ## Arguments
    /// * `request` - The request containing the directory name.
    /// ## Errors
    /// * `Box<dyn Error>` - If the directory name already exists or there is not enough space in the disk.
    fn make_directory(&mut self, request: &MakeDirectoryRequest) -> Void;

    /// Changes the working directory
    /// ## Arguments
    /// * `request` - The request containing the directory name.
    /// ## Errors
    /// * `Box<dyn Error>` - If the directory does not exist.
    fn change_working_directory(&mut self, request: &ChangeDirectoryRequest) -> Void;

    /// Returns the whole path to the working directory
    fn get_working_directory(&self) -> String;

    /// Get boot sector
    fn get_boot_sector(&self) -> &BootSector;

    /// Returns the free space in the disk with respect to the total number of empty clusters
    fn get_free_space(&mut self) -> u64;

    /// Returns the total space in the disk
    fn get_total_space(&self) -> u64;
}
