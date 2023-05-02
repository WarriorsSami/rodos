use crate::application::commands::cd::ChangeDirectoryRequest;
use crate::application::commands::create::CreateRequest;
use crate::application::commands::mkdir::MakeDirectoryRequest;
use crate::application::queries::cat::CatRequest;
use crate::application::Void;
use crate::core::config::Config;
use crate::core::content_type::ContentType;
use crate::core::Arm;
use crate::domain::boot_sector::BootSector;
use crate::domain::fat::{FatTable, FatValue};
use crate::domain::file_entry::{FileEntry, RootTable};
use crate::domain::i_disk_manager::IDiskManager;
use crate::infrastructure::{ByteArray, StorageBuffer};
use crate::CONFIG;
use chrono::Utc;
use std::io::{Read, Write};

/// The `DiskManager` is the main component of the application.
///
/// It is responsible for managing the storage buffer, the FAT table and the root table.
///
/// It is also responsible for the creation of files and directories.
///
/// It is the only component that can access the storage buffer.
///
/// # Fields
/// **`fat`**:
/// - The FAT table is a vector of FatValue.
/// - The FatValue can be `Free`, `Reserved`, `EndOfChain`, 'Bad' or a cluster number.
///
/// **`root`**:
/// - The root table is a vector of FileEntry.
/// - The FileEntry can be a file or a directory.
/// - The FileEntry contains the name, the extension, the size, the attributes and the cluster number.
///
/// **`working_directory`**:
/// - The working directory is a string that represents the current directory.
///
/// **`cluster_size`**:
/// - The cluster size is the size of a cluster in bytes.
///
/// **`cluster_count`**:
/// - The cluster count is the number of clusters in the storage buffer.
///
/// **`storage_buffer`**:
/// - The storage buffer is a vector of vectors of bytes.
/// - The first vector represents the clusters.
/// - The second vector represents the bytes of the cluster.
///
/// **`storage_file_path`**:
/// - The storage file path is the path of the storage file.
///
/// The storage buffer is initialized with the content of the storage file.
/// The FAT table and Root table are initialized with the content of the storage buffer.
#[derive(Debug, Clone)]
pub(crate) struct DiskManager {
    pub(in crate::infrastructure) fat: FatTable,
    pub(in crate::infrastructure) root: RootTable,
    pub(in crate::infrastructure) working_directory: FileEntry,
    pub(in crate::infrastructure) boot_sector: BootSector,
    pub(in crate::infrastructure) storage_buffer: StorageBuffer,
    pub(in crate::infrastructure) storage_file_path: String,
}

impl DiskManager {
    /// Creates a new `DiskManager` based on the configuration and the boot sector provided.
    pub(crate) fn new(config: Arm<Config>, boot_sector: BootSector) -> Self {
        log::info!("Initializing the disk manager...");
        let config = config.lock().expect("Unable to lock config");

        // calculate the number of clusters occupied by the fat table and the root table
        let fat_clusters =
            boot_sector.fat_cell_size * boot_sector.cluster_count / boot_sector.cluster_size;
        let root_clusters = boot_sector.root_entry_cell_size * boot_sector.root_entry_count
            / boot_sector.cluster_size;

        let mut fat = Vec::new();
        fat.resize(boot_sector.cluster_count as usize, FatValue::Free);

        let mut root = Vec::new();
        root.resize(boot_sector.root_entry_count as usize, FileEntry::default());

        // mark the clusters occupied by the boot sector, the fat table and the root table as reserved in the fat table
        for i in 0..boot_sector.clusters_per_boot_sector + fat_clusters + root_clusters {
            fat[i as usize] = FatValue::Reserved;
        }

        // init the storage buffer
        let mut storage_buffer: StorageBuffer = Vec::new();
        storage_buffer.resize(boot_sector.cluster_count as usize, Vec::new());
        storage_buffer
            .iter_mut()
            .for_each(|cluster| cluster.resize(boot_sector.cluster_size as usize, 0));

        Self {
            fat,
            root,
            working_directory: FileEntry::root(),
            boot_sector,
            storage_buffer,
            storage_file_path: config.storage_file_path.clone(),
        }
    }

    /// Initializes the storage buffer with the content of the boot sector, the fat table and the root table
    /// from the in-memory data structures.
    pub(in crate::infrastructure) fn sync_to_buffer(&mut self) {
        // sync boot sector to storage buffer
        let boot_sector_clusters = self.boot_sector.clusters_per_boot_sector as usize;
        let mut boot_sector_data: ByteArray = self.boot_sector.clone().into();

        self.storage_buffer
            .iter_mut()
            .take(boot_sector_clusters)
            .for_each(|cluster| {
                let cluster_size = cluster.len();
                let boot_sector_current_data = boot_sector_data
                    .drain(..cluster_size)
                    .collect::<ByteArray>();

                cluster.copy_from_slice(&boot_sector_current_data);
            });

        // sync fat to storage buffer
        let fat_clusters = self.boot_sector.fat_cell_size as usize * self.fat.len()
            / self.boot_sector.cluster_size as usize;
        let fat_cells_per_cluster = self.boot_sector.cluster_size / self.boot_sector.fat_cell_size;

        self.fat
            // split the fat table into chunks of fat_cells_per_cluster
            .chunks(fat_cells_per_cluster as usize)
            .zip(
                self.storage_buffer
                    .iter_mut()
                    // skip the boot sector clusters from the storage buffer
                    .skip(boot_sector_clusters)
                    .take(fat_clusters),
            )
            // zip the fat table chunks with the storage buffer clusters
            .for_each(|(fat_chunk, cluster)| {
                fat_chunk
                    .iter()
                    .zip(cluster.chunks_mut(self.boot_sector.fat_cell_size as usize))
                    // zip a fat table cell with a section of fat_cell_size bytes from the proper storage buffer cluster
                    .for_each(|(fat_value, chunk)| {
                        let value: u16 = fat_value.clone().into();

                        // split the 2B fat value into 2 chunks of 1B
                        chunk[0] = ((value & 0xFF00) >> 8) as u8;
                        chunk[1] = (value & 0x00FF) as u8;
                    });
            });

        // sync root to storage buffer
        let root_clusters = self.boot_sector.root_entry_cell_size as usize * self.root.len()
            / self.boot_sector.cluster_size as usize;
        let clusters_per_root_entry =
            self.boot_sector.root_entry_cell_size / self.boot_sector.cluster_size;

        self.root
            .iter()
            .zip(
                self.storage_buffer
                    .iter_mut()
                    // skip the boot sector clusters and the fat table clusters from the storage buffer
                    .skip(boot_sector_clusters + fat_clusters)
                    .take(root_clusters)
                    .collect::<Vec<_>>()
                    // split the root table clusters into chunks of clusters_per_root_entry
                    .chunks_mut(clusters_per_root_entry as usize),
            )
            // zip the root table entries with the storage buffer clusters
            .for_each(|(file_entry, cluster)| {
                let mut file_entry_data: ByteArray = file_entry.clone().into();

                let cluster_size = cluster[0].len();
                // extract enough bytes from the file entry to fill the first cluster
                let file_entry_current_data =
                    file_entry_data.drain(..cluster_size).collect::<ByteArray>();

                // copy the extracted bytes to the first cluster
                cluster[0].copy_from_slice(&file_entry_current_data);

                // if there are still bytes left in the file entry, extract them and copy them to the next cluster
                if !file_entry_data.is_empty() {
                    let cluster_size = cluster[1].len();
                    let file_entry_current_data =
                        file_entry_data.drain(..cluster_size).collect::<ByteArray>();

                    cluster[1].copy_from_slice(&file_entry_current_data);
                }
            });
    }

    /// Overwrites the storage file with the content of the storage buffer.
    pub(in crate::infrastructure) fn sync_to_file(&mut self) {
        // sync the storage buffer with the in-memory data structures
        self.sync_to_buffer();

        let mut storage_file =
            std::fs::File::create(&self.storage_file_path).expect("Unable to create storage file");
        self.storage_buffer
            .iter()
            .enumerate()
            .for_each(|(_index, cluster)| {
                storage_file
                    .write_all(cluster)
                    .expect("Unable to write to storage file")
            });

        log::debug!("FAT: {:?}", self.fat);
        log::debug!("Root: {:?}", self.root);
    }

    /// Overwrites the storage buffer with the content of the storage file.
    pub(in crate::infrastructure) fn sync_from_file(&mut self) {
        let mut storage_file =
            std::fs::File::open(&self.storage_file_path).expect("Unable to open storage file");
        self.storage_buffer.iter_mut().for_each(|cluster| {
            storage_file
                .read_exact(cluster)
                .expect("Unable to read from storage file");
        });
    }

    /// Initializes the in-memory data structures from the storage file.
    pub(in crate::infrastructure) fn sync_from_buffer(&mut self, only_boot_sector: bool) {
        // sync storage buffer from file
        self.sync_from_file();

        // sync boot sector from storage buffer
        let boot_sector_clusters = self.boot_sector.clusters_per_boot_sector as usize;
        let boot_sector_data: ByteArray = self
            .storage_buffer
            .iter()
            .take(boot_sector_clusters)
            .flatten()
            .copied()
            .collect();

        self.boot_sector = BootSector::from(boot_sector_data);

        // if only the boot sector is needed, return
        // this is useful when the boot sector is modified and the storage buffer needs to be updated,
        // e.g. after the disk is formatted
        if only_boot_sector {
            return;
        }

        // sync fat from storage buffer
        let fat_clusters = self.boot_sector.fat_cell_size as usize * self.fat.len()
            / self.boot_sector.cluster_size as usize;
        let fat_cells_per_cluster = self.boot_sector.cluster_size / self.boot_sector.fat_cell_size;

        self.storage_buffer
            .iter()
            .skip(boot_sector_clusters)
            .take(fat_clusters)
            .enumerate()
            // zip the fat table chunks with the storage buffer clusters
            .for_each(|(cluster_index, cluster)| {
                // grab the fat table chunk corresponding to the current storage buffer cluster
                let fat_chunk = self
                    .fat
                    .chunks_mut(fat_cells_per_cluster as usize)
                    .nth(cluster_index)
                    .unwrap_or_default();

                cluster
                    .chunks(self.boot_sector.fat_cell_size as usize)
                    .zip(fat_chunk)
                    // zip each fat table cell from the current chunk with the corresponding storage buffer cluster section
                    .for_each(|(chunk, fat_value)| {
                        // convert the fat table cell bytes to a u16 value
                        // the fat table cell is stored in big endian format
                        let value: u16 = ((chunk[0] as u16) << 8) | (chunk[1] as u16);
                        *fat_value = value.into();
                    });
            });

        // sync root from storage buffer
        let clusters_per_root_entry =
            self.boot_sector.root_entry_cell_size / self.boot_sector.cluster_size;

        let root_table: RootTable = self
            .storage_buffer
            .iter()
            .skip(boot_sector_clusters + fat_clusters)
            .cloned()
            .collect::<Vec<_>>()
            // split the root table clusters into chunks of clusters_per_root_entry
            .chunks(clusters_per_root_entry as usize)
            .take(self.root.len())
            .map(|cluster| {
                // check if the current cluster is empty (default uninitialized root entries)
                let cluster_is_empty = cluster[0]
                    .iter()
                    .take((self.boot_sector.root_entry_cell_size / 2) as usize)
                    .all(|byte| *byte == 0);

                match cluster_is_empty {
                    // if the cluster is not empty, parse the serialized file entry from the cluster
                    false => {
                        let mut file_entry_data: ByteArray = Vec::new();

                        file_entry_data.extend_from_slice(&cluster[0]);
                        // check if the current file entry is split across multiple clusters
                        if cluster.len() > 1 {
                            file_entry_data.extend_from_slice(&cluster[1]);
                        }

                        // parse the file entry from the cluster data and deserialize it
                        let mut file_entry_result = FileEntry::from(file_entry_data);
                        // set the parent entry to the root entry
                        file_entry_result.parent_entry = Some(Box::new(FileEntry::root()));

                        // if the file entry is a file, return it
                        if file_entry_result.is_file() {
                            file_entry_result
                        } else {
                            // if the file entry is a directory, iterate over its directory tree
                            // stored in the storage buffer, reconstruct and link it to the current dir entry
                            self.link_root_table_to_directory(&mut file_entry_result);
                            file_entry_result
                        }
                    }
                    // if the cluster is empty, return a default file entry to be used as a placeholder in the root table
                    true => FileEntry::default(),
                }
            })
            .collect::<Vec<_>>();

        // set the root table
        self.root = root_table;
        // propagate any changes from the root table to the storage medium as soon as possible
        self.sync_to_file();

        // init working directory from root
        if self.working_directory.is_root() {
            self.working_directory.parent_entry = None;
            self.working_directory.children_entries = Some(self.root.clone());
        } else {
            self.sync_working_directory_from_root();
        }
    }

    /// Iterate over the root table of a directory and link the directory tree to the current directory entry recursively.
    pub(in crate::infrastructure) fn link_root_table_to_directory(
        &mut self,
        directory_entry: &mut FileEntry,
    ) {
        let mut root_table = RootTable::default();

        // set the head of the allocation chain of the current directory entry
        let mut current_cluster_index = directory_entry.first_cluster;
        while FatValue::from(current_cluster_index) != FatValue::EndOfChain {
            // get the current cluster from the storage buffer relative to the current cluster index
            let mut file_entry_data: ByteArray = self
                .storage_buffer
                .get(current_cluster_index as usize)
                .unwrap()
                .to_vec();

            // check if the file entry is split across multiple clusters
            if self.boot_sector.root_entry_cell_size / self.boot_sector.cluster_size != 1 {
                current_cluster_index = self.fat[current_cluster_index as usize].clone().into();
                let file_entry_data_next_cluster: ByteArray = self
                    .storage_buffer
                    .get(current_cluster_index as usize)
                    .unwrap()
                    .to_vec();
                file_entry_data.extend_from_slice(&file_entry_data_next_cluster);
            }

            // deserialize the file entry from the cluster data and set the parent entry to the current directory entry
            let mut file_entry_result = FileEntry::from(file_entry_data);
            file_entry_result.parent_entry = Some(Box::new(directory_entry.clone()));

            // if the file entry is a file or a special directory entry, return it
            if file_entry_result.is_file()
                || (!file_entry_result.is_file()
                    && (file_entry_result.name == "." || file_entry_result.name == ".."))
            {
                root_table.push(file_entry_result);
            } else {
                // if the file entry is a directory, iterate over its directory tree too in order to
                // reconstruct its directory tree
                self.link_root_table_to_directory(&mut file_entry_result);
                root_table.push(file_entry_result);
            }

            // set the next cluster index
            current_cluster_index = self.fat[current_cluster_index as usize].clone().into();
        }

        // compute the size of the current directory entry
        // as the sum of the size of all the files in the directory and the size of the directory table
        let size_of_file_entries = root_table.iter().map(|entry| entry.size).sum::<u32>();

        directory_entry.size = size_of_file_entries;
        directory_entry.children_entries = Some(root_table.clone());
        // propagate any changes from the root table to the storage medium as soon as possible (dir size in this case)
        self.sync_directory_root_table_to_storage(directory_entry);
    }

    /// Initialize the working directory from the root directory.
    fn sync_working_directory_from_root(&mut self) {
        // get the path from the root to the current working directory
        let path = Self::get_path_from_root_to_entry(&self.working_directory);

        // iterate down the directory tree from the root to the current working directory
        // and set the current working directory to the last entry in the path
        let mut current_entry = self
            .root
            .iter()
            .find(|entry| entry.name == path[1])
            .unwrap();
        for path_part in path.iter().skip(2) {
            let children_entries = current_entry.children_entries.as_ref().unwrap();
            current_entry = children_entries
                .iter()
                .find(|&entry| entry.name == path_part.as_str())
                .unwrap();
        }

        self.working_directory = current_entry.clone();
    }

    /// Get the path from the root to the current working directory.
    fn get_path_from_root_to_entry(entry: &FileEntry) -> Vec<String> {
        let mut path: Vec<String> = Vec::new();
        let mut current_entry = entry.clone();
        path.push(current_entry.name.clone());

        // iterate up the directory tree from the current working directory to the root
        while let Some(parent_entry) = &current_entry.parent_entry {
            let parent_entry = parent_entry.as_ref();
            path.push(parent_entry.name.clone());
            current_entry = parent_entry.clone();
        }

        // reverse the path vector to get the path from the root to the current working directory
        path.reverse();
        path
    }

    /// Serialize the root table of a directory by iterating over its children entries
    /// and concatenating their byte array representations.
    pub(in crate::infrastructure) fn serialize_directory_root_table(
        directory: &FileEntry,
    ) -> Vec<u8> {
        directory
            .children_entries
            .as_ref()
            .unwrap()
            .iter()
            .cloned()
            .flat_map(|file_entry| {
                let file_byte_array: ByteArray = file_entry.into();
                file_byte_array
            })
            .collect::<Vec<u8>>()
    }

    /// Synchronize the root table of a directory to the storage medium.
    pub(in crate::infrastructure) fn sync_directory_root_table_to_storage(
        &mut self,
        dir_entry: &FileEntry,
    ) {
        // free old working directory data
        self.free_clusters(dir_entry);

        // get updated working directory data
        let mut directory_data = Self::serialize_directory_root_table(dir_entry);

        // update fat and storage buffer
        let mut current_cluster_index = dir_entry.first_cluster;

        // while there is still data to be written to the storage
        while !directory_data.is_empty() {
            let mut cluster_data: ByteArray = Vec::new();
            cluster_data
                .extend_from_slice(&directory_data[..self.boot_sector.cluster_size as usize]);
            directory_data.drain(..self.boot_sector.cluster_size as usize);
            // write the cluster data to the current cluster index
            self.storage_buffer[current_cluster_index as usize] = cluster_data.clone();

            let next_cluster_index = self
                .get_next_free_cluster_index_gt(current_cluster_index as usize)
                .unwrap();
            // update the fat with the next cluster index or with the end of chain value
            // depending on whether there is still data to be written to the storage
            self.fat[current_cluster_index as usize] = match directory_data.is_empty() {
                false => FatValue::from(next_cluster_index as u16),
                true => FatValue::EndOfChain,
            };

            // set the current cluster index to the next cluster index
            current_cluster_index = next_cluster_index as u16;
        }
    }

    /// Get the next free cluster index greater than the current cluster index.
    pub(in crate::infrastructure) fn get_next_free_cluster_index_gt(
        &self,
        current_cluster_index: usize,
    ) -> Option<usize> {
        self.fat.iter().enumerate().position(|(index, fat_value)| {
            *fat_value == FatValue::Free && index > current_cluster_index
        })
    }

    /// Iterate over the allocation chain of a file entry and free the clusters associated with it
    /// by setting their fat values to free (the storage remains unchanged).
    pub(in crate::infrastructure) fn free_clusters(&mut self, file_entry: &FileEntry) {
        // delete file entry associated data
        let mut cluster_index = file_entry.first_cluster as usize;
        while self.fat[cluster_index] != FatValue::EndOfChain {
            let next_cluster_index: u16 = self.fat[cluster_index].clone().into();
            self.fat[cluster_index] = FatValue::Free;
            cluster_index = next_cluster_index as usize;
        }
        self.fat[cluster_index] = FatValue::Free;
    }

    /// Delete a file entry from the root table of a directory.
    pub(in crate::infrastructure) fn free_file_entry(&mut self, file_entry: &FileEntry) {
        // delete the actual file entry
        match self.working_directory.is_root() {
            true => {
                let file_entry_index = self.root.iter().position(|entry| {
                    entry.name == file_entry.name && entry.extension == file_entry.extension
                });
                // if in root, set the file entry to default
                if let Some(file_entry_index) = file_entry_index {
                    self.root[file_entry_index] = FileEntry::default();
                }
            }
            false => {
                // otherwise, remove the file entry from the working directory children entries
                self.get_root_table_for_working_directory().retain(|entry| {
                    !(entry.name == file_entry.name && entry.extension == file_entry.extension)
                });
            }
        }
    }

    /// Change the current working directory to the root directory.
    pub(in crate::infrastructure) fn change_working_directory_to_root(&mut self) -> Void {
        while !self.working_directory.is_root() {
            let cd_request = ChangeDirectoryRequest::new("..".to_string());
            self.pull_sync();
            self.change_working_directory(&cd_request)?;
        }

        Ok(())
    }

    /// Change the current working directory to a directory entry.
    pub(in crate::infrastructure) fn change_working_directory_to(
        &mut self,
        dir_entry: &FileEntry,
    ) -> Void {
        self.change_working_directory_to_root()?;

        let path = Self::get_path_from_root_to_entry(dir_entry);

        for path_part in path.iter().skip(1) {
            let cd_request = ChangeDirectoryRequest::new(path_part.clone());
            self.pull_sync();
            self.change_working_directory(&cd_request)?;
        }

        Ok(())
    }

    /// Start from a source directory entry and recursively copy all its children entries to a
    /// destination directory entry (useful for copying directories).
    pub(in crate::infrastructure) fn inflate_directory_tree_inline(
        &mut self,
        src_dir_entry: &FileEntry,
        dest_dir_name: String,
    ) -> Void {
        // change working directory to dir_entry
        let cd_request = ChangeDirectoryRequest::new(dest_dir_name);
        self.pull_sync();
        self.change_working_directory(&cd_request)?;

        // iterate over all children entries
        let root_table = src_dir_entry.children_entries.as_ref();

        if let Some(root_table) = root_table {
            for entry in root_table.iter() {
                if entry.name == "." || entry.name == ".." {
                    continue;
                }

                match entry.is_file() {
                    true => {
                        // save the current working directory
                        let current_working_directory = self.working_directory.clone();

                        // change working directory to src directory
                        self.change_working_directory_to(src_dir_entry)?;

                        // get file content
                        let cat_request =
                            CatRequest::new(entry.name.clone(), entry.extension.clone());
                        let file_content = self.get_file_content(&cat_request)?;

                        // change working directory back to dest directory
                        self.change_working_directory_to(&current_working_directory)?;

                        // write the file content to the temp buffer file
                        DiskManager::write_to_temp_buffer(file_content.as_str())?;

                        // create the file entry
                        let create_request = CreateRequest::new(
                            entry.name.clone(),
                            entry.extension.clone(),
                            file_content.len() as u32,
                            entry.attributes,
                            Utc::now(),
                            ContentType::Temp,
                        );
                        self.create_file(&create_request)?;
                        self.push_sync();
                    }
                    false => {
                        // create the directory entry
                        let make_directory_request = MakeDirectoryRequest::new(
                            entry.name.clone(),
                            entry.attributes,
                            Utc::now(),
                        );
                        self.pull_sync();
                        self.make_directory(&make_directory_request)?;
                        self.push_sync();

                        // iterate over the directory's root table and recreate the dir tree in the new disk representation
                        self.inflate_directory_tree_inline(entry, entry.name.clone())?;
                    }
                }
            }
        }

        // change working directory back to parent
        let cd_request = ChangeDirectoryRequest::new("..".to_string());
        self.pull_sync();
        self.change_working_directory(&cd_request)?;

        Ok(())
    }

    /// Start from a source directory entry and recursively mimic all its children entries to a
    /// destination directory entry from a different in-memory disk representation
    /// (useful for disk defragmentation as it doesn't require to write to disk up to a certain point in order to preserve the consistency).
    pub(in crate::infrastructure) fn inflate_directory_tree(
        &mut self,
        disk_manager: &mut DiskManager,
        dir_entry: &FileEntry,
    ) -> Void {
        // change working directory to dir_entry
        let cd_request = ChangeDirectoryRequest::new(dir_entry.name.clone());
        disk_manager.change_working_directory(&cd_request)?;

        self.pull_sync();
        self.change_working_directory(&cd_request)?;

        // iterate over all children entries
        let root_table = dir_entry.children_entries.as_ref();

        if let Some(root_table) = root_table {
            for entry in root_table.iter() {
                if entry.name == "." || entry.name == ".." {
                    continue;
                }

                match entry.is_file() {
                    true => {
                        // get file content
                        self.pull_sync();
                        let cat_request =
                            CatRequest::new(entry.name.clone(), entry.extension.clone());
                        let file_content = self.get_file_content(&cat_request)?;

                        // write the file content to the temp buffer file
                        DiskManager::write_to_temp_buffer(file_content.as_str())?;

                        // create the file entry
                        let create_request = CreateRequest::new(
                            entry.name.clone(),
                            entry.extension.clone(),
                            file_content.len() as u32,
                            entry.attributes,
                            entry.last_modification_datetime,
                            ContentType::Temp,
                        );
                        disk_manager.create_file(&create_request)?;
                    }
                    false => {
                        // create the directory entry
                        let make_directory_request = MakeDirectoryRequest::new(
                            entry.name.clone(),
                            entry.attributes,
                            entry.last_modification_datetime,
                        );
                        disk_manager.make_directory(&make_directory_request)?;

                        // iterate over the directory's root table and recreate the dir tree in the new disk representation
                        self.inflate_directory_tree(disk_manager, entry)?;
                    }
                }
            }
        }

        // change working directory back to parent
        let cd_request = ChangeDirectoryRequest::new("..".to_string());
        disk_manager.change_working_directory(&cd_request)?;

        self.pull_sync();
        self.change_working_directory(&cd_request)?;

        Ok(())
    }

    /// Allocate a new cluster chain in the FAT and write the file data to the storage buffer
    /// for a given file entry.
    pub(in crate::infrastructure) fn write_data_to_disk(
        &mut self,
        file_entry: &FileEntry,
        file_data: &mut Vec<u8>,
    ) -> Void {
        // create the cluster chain in fat and write the file data to the storage buffer
        let mut current_cluster_index = file_entry.first_cluster as usize;
        let mut remaining_file_size = file_entry.size as u16;

        // while there is still data to write
        while remaining_file_size > 0 {
            match self.get_next_free_cluster_index_gt(current_cluster_index) {
                Some(next_cluster_index) => {
                    // mark the current cluster as data and point it to the next cluster
                    self.fat[current_cluster_index] = FatValue::Data(next_cluster_index as u16);

                    // fill the current cluster with data extracted from the file data
                    self.storage_buffer[current_cluster_index] = file_data
                        .drain(
                            ..std::cmp::min(
                                self.boot_sector.cluster_size as usize,
                                remaining_file_size as usize,
                            ),
                        )
                        .collect();

                    if remaining_file_size > self.boot_sector.cluster_size {
                        // update the remaining file size and the current cluster index
                        remaining_file_size -= self.boot_sector.cluster_size;
                        current_cluster_index = next_cluster_index;
                    } else {
                        // add the remaining padding as 0 at the end of the cluster
                        self.storage_buffer[current_cluster_index]
                            .resize(self.boot_sector.cluster_size as usize, 0);
                        self.fat[current_cluster_index] = FatValue::EndOfChain;
                        remaining_file_size = 0;
                    }
                }
                None => {
                    // mark the current cluster as end of chain and free the chain cluster and the file entry
                    self.fat[current_cluster_index] = FatValue::EndOfChain;
                    self.free_clusters(file_entry);
                    self.free_file_entry(file_entry);
                    self.sync_directory_root_table_to_storage(&self.working_directory.clone());

                    return Err(Box::try_from("No space in fat".to_string()).unwrap());
                }
            }
        }

        Ok(())
    }

    /// Write the file data to the temp buffer file in order to be read its data later when
    /// recreating the file entry in the new disk representation after defragmentation.
    pub(in crate::infrastructure) fn write_to_temp_buffer(file_content: &str) -> Void {
        let mut temp_file = std::fs::File::create(CONFIG.temp_file_path.clone())?;
        temp_file.write_all(file_content.as_bytes())?;

        Ok(())
    }

    /// Return the root table of the working directory.
    pub(in crate::infrastructure) fn get_root_table_for_working_directory(
        &mut self,
    ) -> &mut RootTable {
        match self.working_directory.is_root() {
            true => &mut self.root,
            false => self.working_directory.children_entries.as_mut().unwrap(),
        }
    }

    /// Attach a new file entry to the root table of the working directory.
    pub(in crate::infrastructure) fn append_to_root_table_of_working_dir(
        &mut self,
        file_entry: FileEntry,
    ) -> Void {
        match self.working_directory.is_root() {
            true => {
                let dir_file_entry_index = self
                    .root
                    .iter()
                    .position(|file_entry| file_entry.name.is_empty());

                // if the working directory is the root, just append the file entry to the root table
                if let Some(dir_file_entry_index) = dir_file_entry_index {
                    self.root[dir_file_entry_index] = file_entry;
                    Ok(())
                } else {
                    Err(Box::try_from("No space left in the root folder").unwrap())
                }
            }
            false => {
                // add to root table
                self.working_directory
                    .children_entries
                    .as_mut()
                    .unwrap()
                    .push(file_entry.clone());

                // get to the last cluster in the allocation chain of the working directory's root table
                let mut current_cluster_index = self.working_directory.first_cluster;
                while self.fat[current_cluster_index as usize] != FatValue::EndOfChain {
                    current_cluster_index = self.fat[current_cluster_index as usize].clone().into();
                }

                // serialize the file entry and write it to the storage buffer
                let mut file_entry_data: ByteArray = file_entry.into();
                let mut next_cluster_index = self.get_next_free_cluster_index_gt(0).unwrap();
                self.fat[current_cluster_index as usize] =
                    FatValue::from(next_cluster_index as u16);

                // while there is still data to write
                while !file_entry_data.is_empty() {
                    let mut cluster_data: ByteArray = Vec::new();
                    cluster_data.extend_from_slice(
                        &file_entry_data[..self.boot_sector.cluster_size as usize],
                    );
                    file_entry_data.drain(..self.boot_sector.cluster_size as usize);
                    // extract from the file entry data the data that will be written to the current cluster
                    self.storage_buffer[next_cluster_index] = cluster_data.clone();

                    // if there is still data to write, get the next free cluster index and point the current cluster to it
                    // in the fat table
                    current_cluster_index = next_cluster_index as u16;
                    next_cluster_index = self
                        .get_next_free_cluster_index_gt(current_cluster_index as usize)
                        .unwrap();
                    self.fat[current_cluster_index as usize] =
                        FatValue::from(next_cluster_index as u16);
                }
                // mark the last cluster as end of chain
                self.fat[current_cluster_index as usize] = FatValue::EndOfChain;

                Ok(())
            }
        }
    }
}
