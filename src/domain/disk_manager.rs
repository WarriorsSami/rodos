use crate::application::create::CreateRequest;
use crate::application::Void;
use crate::core::content_type::ContentGenerator;
use crate::core::Arm;
use crate::domain::config::Config;
use crate::domain::fat::{FatTable, FatValue};
use crate::domain::file_entry::{FileEntry, FileEntryAttributes, RootTable};
use crate::domain::i_disk_manager::IDiskManager;
use std::io::{Read, Write};

pub(crate) type ByteArray = Vec<u8>;
pub(crate) type StorageBuffer = Vec<ByteArray>;

#[derive(Debug, Clone)]
pub(crate) struct DiskManager {
    pub(crate) fat: FatTable,
    pub(crate) root: RootTable,
    pub(crate) current_dir: String,
    pub(crate) cluster_size: u32,
    pub(crate) cluster_count: u32,
    pub(crate) storage_buffer: StorageBuffer,
    pub(crate) storage_file_path: String,
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
            current_dir: "/".to_string(),
            cluster_size: config.cluster_size,
            cluster_count: config.cluster_count,
            storage_buffer,
            storage_file_path: config.storage_file_path.clone(),
        }
    }

    fn sync_to_buffer(&mut self) {
        // sync fat to storage buffer
        let fat_cells_per_cluster = self.cluster_size / 2;
        self.fat
            .chunks(fat_cells_per_cluster as usize)
            .enumerate()
            .for_each(|(cluster_index, fat_chunk)| {
                let mut cluster = Vec::new();
                cluster.resize(self.cluster_size as usize, 0);

                fat_chunk
                    .iter()
                    .zip(cluster.chunks_mut(2))
                    .for_each(|(fat_value, chunk)| {
                        let value: u16 = fat_value.clone().into();

                        // split the 2B fat value into 2 chunks of 1B
                        chunk[0] = ((value & 0xFF00) >> 8) as u8;
                        chunk[1] = (value & 0x00FF) as u8;
                    });

                self.storage_buffer[cluster_index] = cluster;
            });

        // sync root to storage buffer
        let fat_clusters = 2 * self.fat.len() / self.cluster_size as usize;
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
                .expect("Unable to read from storage file")
        });
    }

    fn sync_from_buffer(&mut self) {
        self.sync_from_file();

        // sync fat from storage buffer
        let fat_cells_per_cluster = self.cluster_size / 2;
        self.storage_buffer
            .iter()
            .take(self.fat.len())
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
            .skip(self.fat.len())
            .zip(self.root.iter_mut())
            .for_each(|(cluster, file_entry)| {
                let file_entry_data: ByteArray = file_entry.clone().into();
                *file_entry = FileEntry::from(cluster[..file_entry_data.len()].to_vec());
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
            return Err(Box::try_from("File already exists".to_string()).unwrap());
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
            == required_clusters
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
        let file_entry = FileEntry {
            name: request.file_name,
            extension: request.file_extension,
            size: request.file_size,
            first_cluster: first_cluster as u32,
            attributes: FileEntryAttributes::File as u8,
        };
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
                .position(|fat_value| *fat_value == FatValue::Free)
            {
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

                    current_cluster = next_cluster;
                    if remaining_file_size > self.cluster_size {
                        remaining_file_size -= self.cluster_size;
                    } else {
                        remaining_file_size = 0;
                    }
                }
                None => {
                    self.free_clusters_and_entry(&file_entry);
                    return Err(Box::try_from("No space in fat".to_string()).unwrap());
                }
            }
        }
        self.fat[current_cluster] = FatValue::EndOfChain;

        // println!("{:?}", self.fat);

        // push sync
        self.push_sync();

        Ok(())
    }
}
