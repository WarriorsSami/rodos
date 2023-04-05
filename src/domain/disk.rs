use crate::domain::config::Config;
use crate::domain::fat::{FatTable, FatValue};
use crate::domain::file_entry::{FileEntry, RootTable};
use std::io::Write;

pub(crate) type StorageBuffer = Vec<Vec<u8>>;

#[derive(Debug, Clone)]
pub(crate) struct Disk {
    pub(crate) fat: FatTable,
    pub(crate) root: RootTable,
    pub(crate) current_dir: String,
    pub(crate) cluster_size: u32,
    pub(crate) cluster_count: u32,
    pub(crate) storage_buffer: StorageBuffer,
    pub(crate) storage_file_path: String,
}

impl Disk {
    pub(crate) fn new(config: Config) -> Self {
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
            storage_file_path: config.storage_file_path,
        }
    }

    pub(crate) fn sync_to_buffer(&mut self) {
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
                        chunk[0] = (value & 0x00FF) as u8;
                        chunk[1] = ((value & 0xFF00) >> 8) as u8;
                    });

                self.storage_buffer[cluster_index] = cluster;
            });

        // sync root to storage buffer
        self.root
            .iter()
            .zip(self.storage_buffer.iter_mut().skip(self.fat.len()))
            .for_each(|(file_entry, cluster)| {
                let file_entry_data: Vec<u8> = file_entry.clone().into();
                cluster[..file_entry_data.len()].clone_from_slice(&file_entry_data);
            });
    }

    pub(crate) fn sync_to_file(&mut self) {
        self.sync_to_buffer();

        let mut storage_file =
            std::fs::File::create(&self.storage_file_path).expect("Unable to create storage file");
        self.storage_buffer.iter().for_each(|cluster| {
            storage_file
                .write_all(cluster)
                .expect("Unable to write to storage file")
        });
    }
}
