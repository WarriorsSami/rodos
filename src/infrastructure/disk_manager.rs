use crate::application::create::CreateRequest;
use crate::application::rename::RenameRequest;
use crate::application::Void;
use crate::core::content_type::ContentGenerator;
use crate::core::Arm;
use crate::domain::config::Config;
use crate::domain::fat::{FatTable, FatValue};
use crate::domain::file_entry::{FileEntry, FileEntryAttributes, RootTable};
use crate::domain::i_disk_manager::IDiskManager;
use std::error::Error;
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
    fat: FatTable,
    root: RootTable,
    working_directory: String,
    cluster_size: u32,
    cluster_count: u32,
    storage_buffer: StorageBuffer,
    storage_file_path: String,
}

impl DiskManager {
    pub(crate) fn new(config: Arm<Config>) -> Self {
        let config = config.lock().unwrap();

        let fat_clusters = 2 * config.cluster_count / config.cluster_size;
        let root_clusters = 64;

        let mut fat = Vec::new();
        fat.resize(config.cluster_count as usize, FatValue::Free);

        let mut root = Vec::new();
        root.resize(root_clusters as usize, FileEntry::default());

        for i in 0..fat_clusters + root_clusters {
            fat[i as usize] = FatValue::Reserved;
        }

        let mut storage_buffer: StorageBuffer = Vec::new();
        storage_buffer.resize(config.cluster_count as usize, Vec::new());
        storage_buffer
            .iter_mut()
            .for_each(|cluster| cluster.resize(config.cluster_size as usize, 0));

        Self {
            fat,
            root,
            working_directory: "/".to_string(),
            cluster_size: config.cluster_size,
            cluster_count: config.cluster_count,
            storage_buffer,
            storage_file_path: config.storage_file_path.clone(),
        }
    }

    fn sync_to_buffer(&mut self) {
        // sync fat to storage buffer
        let fat_clusters = 2 * self.fat.len() / self.cluster_size as usize;
        let fat_cells_per_cluster = self.cluster_size / 2;

        self.fat
            .chunks(fat_cells_per_cluster as usize)
            .zip(self.storage_buffer.iter_mut().take(fat_clusters))
            .for_each(|(fat_chunk, cluster)| {
                fat_chunk
                    .iter()
                    .zip(cluster.chunks_mut(2))
                    .for_each(|(fat_value, chunk)| {
                        let value: u16 = fat_value.clone().into();

                        // split the 2B fat value into 2 chunks of 1B
                        chunk[0] = ((value & 0xFF00) >> 8) as u8;
                        chunk[1] = (value & 0x00FF) as u8;
                    });
            });

        // sync root to storage buffer
        self.root
            .iter()
            .zip(self.storage_buffer.iter_mut().skip(fat_clusters))
            .for_each(|(file_entry, cluster)| {
                let file_entry_data: ByteArray = file_entry.clone().into();
                cluster[..file_entry_data.len()].clone_from_slice(&file_entry_data);
            });
    }

    fn sync_to_file(&mut self) {
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
    }

    fn sync_from_file(&mut self) {
        let mut storage_file =
            std::fs::File::open(&self.storage_file_path).expect("Unable to open storage file");
        self.storage_buffer.iter_mut().for_each(|cluster| {
            storage_file
                .read_exact(cluster)
                .expect("Unable to read from storage file");
        });
    }

    fn sync_from_buffer(&mut self) {
        self.sync_from_file();

        // sync fat from storage buffer
        let fat_clusters = 2 * self.fat.len() / self.cluster_size as usize;
        let fat_cells_per_cluster = self.cluster_size / 2;

        self.storage_buffer
            .iter()
            .take(fat_clusters)
            .enumerate()
            .for_each(|(cluster_index, cluster)| {
                let fat_chunk = self
                    .fat
                    .chunks_mut(fat_cells_per_cluster as usize)
                    .nth(cluster_index)
                    .unwrap();

                cluster
                    .chunks(2)
                    .zip(fat_chunk)
                    .for_each(|(chunk, fat_value)| {
                        let value: u16 = ((chunk[0] as u16) << 8) | (chunk[1] as u16);
                        *fat_value = value.into();
                    });
            });

        // sync root from storage buffer
        self.storage_buffer
            .iter()
            .skip(fat_clusters)
            .zip(self.root.iter_mut())
            .for_each(|(cluster, file_entry)| {
                let file_entry_data: ByteArray = file_entry.clone().into();
                let cluster_is_empty = cluster[..file_entry_data.len()]
                    .iter()
                    .all(|byte| *byte == 0);

                *file_entry = match cluster_is_empty {
                    false => FileEntry::from(cluster[..file_entry_data.len()].to_owned()),
                    true => FileEntry::default(),
                };
            });
    }

    fn free_clusters_and_entry(&mut self, file_entry: &FileEntry) {
        let mut cluster_index = file_entry.first_cluster as usize;
        while self.fat[cluster_index] != FatValue::EndOfChain {
            let next_cluster_index: u16 = self.fat[cluster_index].clone().into();
            self.fat[cluster_index] = FatValue::Free;
            cluster_index = next_cluster_index as usize;
        }
        self.fat[cluster_index] = FatValue::Free;

        let file_entry_index = self
            .root
            .iter()
            .position(|entry| {
                entry.name == file_entry.name && entry.extension == file_entry.extension
            })
            .unwrap();
        self.root[file_entry_index] = FileEntry::default();
    }
}

impl IDiskManager for DiskManager {
    fn push_sync(&mut self) {
        self.sync_to_file();
    }

    fn pull_sync(&mut self) {
        self.sync_from_buffer();
    }

    fn create_file(&mut self, request: CreateRequest) -> Void {
        // check if file already exists in root
        if self.root.iter().any(|file_entry| {
            file_entry.name == request.file_name && file_entry.extension == request.file_extension
        }) {
            return Err(Box::try_from(format!(
                "File {}.{} already exists",
                request.file_name, request.file_extension
            ))
            .unwrap());
        }

        // check if there is enough space in root
        if self
            .root
            .iter()
            .all(|file_entry| !file_entry.name.is_empty())
        {
            return Err(Box::try_from("No space in root".to_string()).unwrap());
        }

        // check if there is enough space in fat
        let required_clusters =
            (request.file_size as f64 / self.cluster_size as f64).ceil() as usize;
        if self
            .fat
            .iter()
            .filter(|&fat_value| *fat_value == FatValue::Free)
            .count()
            < required_clusters
        {
            return Err(Box::try_from("No space in fat".to_string()).unwrap());
        }

        // find a free cluster in fat
        let first_cluster = self
            .fat
            .iter()
            .position(|fat_value| *fat_value == FatValue::Free)
            .unwrap();

        // create file entry in root
        let file_entry = FileEntry::new(
            request.file_name,
            request.file_extension,
            request.file_size,
            first_cluster as u32,
            FileEntryAttributes::File as u8,
        );
        let file_entry_index = self
            .root
            .iter()
            .position(|file_entry| file_entry.name.is_empty())
            .unwrap();

        self.root[file_entry_index] = file_entry.clone();

        // create the cluster chain in fat and write the file data to the storage buffer
        let mut current_cluster = first_cluster;
        let mut remaining_file_size = request.file_size;
        let mut file_data = ContentGenerator::generate(request.content_type, request.file_size);

        while remaining_file_size > 0 {
            match self
                .fat
                .iter()
                .enumerate()
                .position(|(cluster_index, fat_value)| {
                    *fat_value == FatValue::Free && cluster_index > current_cluster
                }) {
                Some(next_cluster) => {
                    self.fat[current_cluster] = FatValue::Data(next_cluster as u32);

                    self.storage_buffer[current_cluster] = file_data
                        .drain(
                            ..std::cmp::min(
                                self.cluster_size as usize,
                                remaining_file_size as usize,
                            ),
                        )
                        .collect();

                    if remaining_file_size > self.cluster_size {
                        remaining_file_size -= self.cluster_size;
                        current_cluster = next_cluster;
                    } else {
                        // add the remaining padding as 0 at the end of the cluster
                        self.storage_buffer[current_cluster].resize(self.cluster_size as usize, 0);
                        self.fat[current_cluster] = FatValue::EndOfChain;
                        remaining_file_size = 0;
                    }
                }
                None => {
                    // mark the current cluster as end of chain and free the chain cluster and the file entry
                    self.fat[current_cluster] = FatValue::EndOfChain;
                    self.free_clusters_and_entry(&file_entry);
                    return Err(Box::try_from("No space in fat".to_string()).unwrap());
                }
            }
        }

        // push sync
        self.push_sync();

        Ok(())
    }

    fn list_files(&mut self) -> Result<RootTable, Box<dyn Error>> {
        self.pull_sync();

        // filter away empty entries
        let root = self
            .root
            .iter()
            .filter(|&file_entry| !file_entry.name.is_empty())
            .cloned()
            .collect();

        Ok(root)
    }

    fn rename_file(&mut self, request: RenameRequest) -> Void {
        // check if the old file exists in root
        if !self.root.iter().any(|file_entry| {
            file_entry.name == request.old_name && file_entry.extension == request.old_extension
        }) {
            return Err(Box::try_from(format!(
                "File {}.{} does not exist",
                request.old_name, request.old_extension
            ))
            .unwrap());
        }

        // check if a file with the new name already exists in root
        if self.root.iter().any(|file_entry| {
            file_entry.name == request.new_name && file_entry.extension == request.new_extension
        }) {
            return Err(Box::try_from(format!(
                "File {}.{} already exists",
                request.new_name, request.new_extension
            ))
            .unwrap());
        }

        // rename the file in root
        let file_entry_index = self
            .root
            .iter()
            .position(|file_entry| {
                file_entry.name == request.old_name && file_entry.extension == request.old_extension
            })
            .unwrap();

        self.root[file_entry_index].name = request.new_name;
        self.root[file_entry_index].extension = request.new_extension;

        // push sync
        self.push_sync();

        Ok(())
    }

    fn get_working_directory(&self) -> String {
        self.working_directory.clone()
    }

    fn get_free_space(&mut self) -> u64 {
        self.pull_sync();

        let free_clusters = self
            .fat
            .iter()
            .filter(|&fat_value| *fat_value == FatValue::Free)
            .count();

        (free_clusters * self.cluster_size as usize) as u64
    }

    fn get_total_space(&self) -> u64 {
        (self.fat.len() * self.cluster_size as usize) as u64
    }
}
