use crate::application::cat::CatRequest;
use crate::application::cd::ChangeDirectoryRequest;
use crate::application::create::CreateRequest;
use crate::application::mkdir::MakeDirectoryRequest;
use crate::application::Void;
use crate::core::config::Config;
use crate::core::content_type::ContentType;
use crate::core::Arm;
use crate::domain::boot_sector::BootSector;
use crate::domain::fat::{FatTable, FatValue};
use crate::domain::file_entry::{FileEntry, RootTable};
use crate::domain::i_disk_manager::IDiskManager;
use crate::CONFIG;
use std::io::{Read, Write};

pub(crate) type ByteArray = Vec<u8>;
pub(crate) type StorageBuffer = Vec<ByteArray>;

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
/// The FAT table is initialized with the content of the storage buffer.
/// The root table is initialized with the content of the FAT table.
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
    pub(crate) fn new(config: Arm<Config>, boot_sector: BootSector) -> Self {
        log::info!("Initializing the disk manager...");
        let config = config.lock().expect("Unable to lock config");

        let fat_clusters =
            boot_sector.fat_cell_size * boot_sector.cluster_count / boot_sector.cluster_size;
        let root_clusters = boot_sector.root_entry_cell_size * boot_sector.root_entry_count
            / boot_sector.cluster_size;

        let mut fat = Vec::new();
        fat.resize(boot_sector.cluster_count as usize, FatValue::Free);

        let mut root = Vec::new();
        root.resize(boot_sector.root_entry_count as usize, FileEntry::default());

        for i in 0..boot_sector.clusters_per_boot_sector + fat_clusters + root_clusters {
            fat[i as usize] = FatValue::Reserved;
        }

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
            .chunks(fat_cells_per_cluster as usize)
            .zip(
                self.storage_buffer
                    .iter_mut()
                    .skip(boot_sector_clusters)
                    .take(fat_clusters),
            )
            .for_each(|(fat_chunk, cluster)| {
                fat_chunk
                    .iter()
                    .zip(cluster.chunks_mut(self.boot_sector.fat_cell_size as usize))
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
                    .skip(boot_sector_clusters + fat_clusters)
                    .take(root_clusters)
                    .collect::<Vec<_>>()
                    .chunks_mut(clusters_per_root_entry as usize),
            )
            .for_each(|(file_entry, cluster)| {
                let mut file_entry_data: ByteArray = file_entry.clone().into();

                let cluster_size = cluster[0].len();
                let file_entry_current_data =
                    file_entry_data.drain(..cluster_size).collect::<ByteArray>();

                cluster[0].copy_from_slice(&file_entry_current_data);

                if !file_entry_data.is_empty() {
                    let cluster_size = cluster[1].len();
                    let file_entry_current_data =
                        file_entry_data.drain(..cluster_size).collect::<ByteArray>();

                    cluster[1].copy_from_slice(&file_entry_current_data);
                }
            });
    }

    pub(in crate::infrastructure) fn sync_to_file(&mut self) {
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

    pub(in crate::infrastructure) fn sync_from_file(&mut self) {
        let mut storage_file =
            std::fs::File::open(&self.storage_file_path).expect("Unable to open storage file");
        self.storage_buffer.iter_mut().for_each(|cluster| {
            storage_file
                .read_exact(cluster)
                .expect("Unable to read from storage file");
        });
    }

    pub(in crate::infrastructure) fn sync_from_buffer(&mut self, only_boot_sector: bool) {
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
            .for_each(|(cluster_index, cluster)| {
                let fat_chunk = self
                    .fat
                    .chunks_mut(fat_cells_per_cluster as usize)
                    .nth(cluster_index)
                    .unwrap_or_default();

                cluster
                    .chunks(self.boot_sector.fat_cell_size as usize)
                    .zip(fat_chunk)
                    .for_each(|(chunk, fat_value)| {
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
            .chunks(clusters_per_root_entry as usize)
            .take(self.root.len())
            .map(|cluster| {
                let cluster_is_empty = cluster[0]
                    .iter()
                    .take((self.boot_sector.root_entry_cell_size / 2) as usize)
                    .all(|byte| *byte == 0);

                match cluster_is_empty {
                    false => {
                        let mut file_entry_data: ByteArray = Vec::new();

                        file_entry_data.extend_from_slice(&cluster[0]);
                        if cluster.len() > 1 {
                            file_entry_data.extend_from_slice(&cluster[1]);
                        }

                        let mut file_entry_result = FileEntry::from(file_entry_data);
                        file_entry_result.parent_entry = Some(Box::new(FileEntry::root()));

                        if file_entry_result.is_file() {
                            file_entry_result
                        } else {
                            self.link_root_table_to_directory(&mut file_entry_result);
                            file_entry_result
                        }
                    }
                    true => FileEntry::default(),
                }
            })
            .collect::<Vec<_>>();

        self.root = root_table;
        self.sync_to_file();

        if self.working_directory.name == "/" {
            self.working_directory.parent_entry = None;
            self.working_directory.children_entries = Some(self.root.clone());
        } else {
            self.sync_working_directory_from_root();
        }
    }

    pub(in crate::infrastructure) fn link_root_table_to_directory(
        &mut self,
        directory_entry: &mut FileEntry,
    ) {
        let mut root_table = RootTable::default();

        let mut current_cluster_index = directory_entry.first_cluster;
        while FatValue::from(current_cluster_index) != FatValue::EndOfChain {
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

            let mut file_entry_result = FileEntry::from(file_entry_data);
            file_entry_result.parent_entry = Some(Box::new(directory_entry.clone()));

            if file_entry_result.is_file()
                || (!file_entry_result.is_file()
                    && (file_entry_result.name == "." || file_entry_result.name == ".."))
            {
                root_table.push(file_entry_result);
            } else {
                self.link_root_table_to_directory(&mut file_entry_result);
                root_table.push(file_entry_result);
            }

            current_cluster_index = self.fat[current_cluster_index as usize].clone().into();
        }

        let cnt_dirs = root_table.iter().filter(|entry| !entry.is_file()).count() as u16;
        let size_of_files = root_table
            .iter()
            .filter(|entry| entry.is_file())
            .map(|entry| entry.size)
            .sum::<u32>();

        directory_entry.size =
            size_of_files + (self.boot_sector.root_entry_cell_size * cnt_dirs) as u32;
        directory_entry.children_entries = Some(root_table.clone());
        self.sync_directory_root_table_to_storage(directory_entry);
    }

    fn sync_working_directory_from_root(&mut self) {
        let path = Self::get_path_from_root_to_entry(&self.working_directory);

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

    fn get_path_from_root_to_entry(entry: &FileEntry) -> Vec<String> {
        let mut path: Vec<String> = Vec::new();
        let mut current_entry = entry.clone();
        path.push(current_entry.name.clone());

        while let Some(parent_entry) = &current_entry.parent_entry {
            let parent_entry = parent_entry.as_ref();
            path.push(parent_entry.name.clone());
            current_entry = parent_entry.clone();
        }

        path.reverse();
        path
    }

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

        while !directory_data.is_empty() {
            let mut cluster_data: ByteArray = Vec::new();
            cluster_data
                .extend_from_slice(&directory_data[..self.boot_sector.cluster_size as usize]);
            directory_data.drain(..self.boot_sector.cluster_size as usize);
            self.storage_buffer[current_cluster_index as usize] = cluster_data.clone();

            let next_cluster_index = self
                .get_next_free_cluster_index_gt(current_cluster_index as usize)
                .unwrap();
            self.fat[current_cluster_index as usize] = match directory_data.is_empty() {
                false => FatValue::from(next_cluster_index as u16),
                true => FatValue::EndOfChain,
            };

            current_cluster_index = next_cluster_index as u16;
        }
    }

    pub(in crate::infrastructure) fn get_next_free_cluster_index_gt(
        &self,
        current_cluster_index: usize,
    ) -> Option<usize> {
        self.fat.iter().enumerate().position(|(index, fat_value)| {
            *fat_value == FatValue::Free && index > current_cluster_index
        })
    }

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

    pub(in crate::infrastructure) fn free_file_entry(&mut self, file_entry: &FileEntry) {
        // delete the actual file entry
        match self.working_directory.name == "/" {
            true => {
                let file_entry_index = self.root.iter().position(|entry| {
                    entry.name == file_entry.name && entry.extension == file_entry.extension
                });
                if let Some(file_entry_index) = file_entry_index {
                    self.root[file_entry_index] = FileEntry::default();
                }
            }
            false => {
                self.get_root_table_for_working_directory().retain(|entry| {
                    !(entry.name == file_entry.name && entry.extension == file_entry.extension)
                });
                self.sync_directory_root_table_to_storage(&self.working_directory.clone());
            }
        }
    }

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

    pub(in crate::infrastructure) fn write_to_temp_buffer(file_content: &str) -> Void {
        let mut temp_file = std::fs::File::create(CONFIG.temp_file_path.clone())?;
        temp_file.write_all(file_content.as_bytes())?;

        Ok(())
    }

    pub(in crate::infrastructure) fn get_root_table_for_working_directory(
        &mut self,
    ) -> &mut RootTable {
        match self.working_directory.name == "/" {
            true => &mut self.root,
            false => self.working_directory.children_entries.as_mut().unwrap(),
        }
    }

    pub(in crate::infrastructure) fn append_to_root_table_of_working_dir(
        &mut self,
        file_entry: FileEntry,
    ) -> Void {
        match self.working_directory.name == "/" {
            true => {
                let dir_file_entry_index = self
                    .root
                    .iter()
                    .position(|file_entry| file_entry.name.is_empty());

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

                // add to storage buffer
                let mut current_cluster_index = self.working_directory.first_cluster;
                while self.fat[current_cluster_index as usize] != FatValue::EndOfChain {
                    current_cluster_index = self.fat[current_cluster_index as usize].clone().into();
                }

                let mut file_entry_data: ByteArray = file_entry.into();
                let mut next_cluster_index = self.get_next_free_cluster_index_gt(0).unwrap();
                self.fat[current_cluster_index as usize] =
                    FatValue::from(next_cluster_index as u16);

                while !file_entry_data.is_empty() {
                    let mut cluster_data: ByteArray = Vec::new();
                    cluster_data.extend_from_slice(
                        &file_entry_data[..self.boot_sector.cluster_size as usize],
                    );
                    file_entry_data.drain(..self.boot_sector.cluster_size as usize);
                    self.storage_buffer[next_cluster_index] = cluster_data.clone();

                    current_cluster_index = next_cluster_index as u16;
                    next_cluster_index = self
                        .get_next_free_cluster_index_gt(current_cluster_index as usize)
                        .unwrap();
                    self.fat[current_cluster_index as usize] =
                        FatValue::from(next_cluster_index as u16);
                }
                self.fat[current_cluster_index as usize] = FatValue::EndOfChain;

                Ok(())
            }
        }
    }
}
