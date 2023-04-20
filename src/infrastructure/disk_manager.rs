use crate::application::cat::CatRequest;
use crate::application::cp::CopyRequest;
use crate::application::create::CreateRequest;
use crate::application::del::DeleteRequest;
use crate::application::fmt::FormatRequest;
use crate::application::ls::ListRequest;
use crate::application::rename::RenameRequest;
use crate::application::Void;
use crate::core::config::Config;
use crate::core::content_type::{ContentGenerator, ContentType};
use crate::core::filter_type::FilterType;
use crate::core::sort_type::SortType;
use crate::core::Arm;
use crate::domain::boot_sector::BootSector;
use crate::domain::fat::{FatTable, FatValue};
use crate::domain::file_entry::{FileEntry, FileEntryAttributes, RootTable};
use crate::domain::i_disk_manager::IDiskManager;
use crate::{CONFIG, CONFIG_ARC};
use chrono::Utc;
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
    working_directory: FileEntry,
    boot_sector: BootSector,
    storage_buffer: StorageBuffer,
    storage_file_path: String,
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

    fn sync_to_buffer(&mut self) {
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

        log::debug!("FAT: {:?}", self.fat);
        log::debug!("Root: {:?}", self.root);
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

    fn sync_from_buffer(&mut self, only_boot_sector: bool) {
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

        self.storage_buffer
            .iter()
            .skip(boot_sector_clusters + fat_clusters)
            .collect::<Vec<_>>()
            .chunks(clusters_per_root_entry as usize)
            .zip(self.root.iter_mut())
            .for_each(|(cluster, file_entry)| {
                let cluster_is_empty = cluster[0].iter().all(|byte| *byte == 0);

                *file_entry = match cluster_is_empty {
                    false => {
                        let mut file_entry_data: ByteArray = Vec::new();

                        file_entry_data.extend_from_slice(cluster[0]);
                        if cluster.len() > 1 {
                            file_entry_data.extend_from_slice(cluster[1]);
                        }

                        FileEntry::from(file_entry_data)
                    }
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

    fn write_to_temp(file_content: &str) -> Void {
        let mut temp_file = std::fs::File::create(CONFIG.temp_file_path.clone())?;
        temp_file.write_all(file_content.as_bytes())?;

        Ok(())
    }
}

impl IDiskManager for DiskManager {
    fn push_sync(&mut self) {
        self.sync_to_file();
    }

    fn pull_sync(&mut self) {
        self.sync_from_buffer(false);
    }

    fn pull_boot_sector_sync(&mut self) {
        self.sync_from_buffer(true);
    }

    fn create_file(&mut self, request: &CreateRequest) -> Void {
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
            (request.file_size as f64 / self.boot_sector.cluster_size as f64).ceil() as usize;
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
            request.file_name.to_owned(),
            request.file_extension.to_owned(),
            request.file_size.to_owned(),
            first_cluster as u16,
            FileEntryAttributes::File as u8,
            Utc::now(),
        );
        let file_entry_index = self
            .root
            .iter()
            .position(|file_entry| file_entry.name.is_empty())
            .unwrap();

        self.root[file_entry_index] = file_entry.clone();

        // create the cluster chain in fat and write the file data to the storage buffer
        let mut current_cluster = first_cluster;
        let mut remaining_file_size = request.file_size as u16;
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
                    self.fat[current_cluster] = FatValue::Data(next_cluster as u16);

                    self.storage_buffer[current_cluster] = file_data
                        .drain(
                            ..std::cmp::min(
                                self.boot_sector.cluster_size as usize,
                                remaining_file_size as usize,
                            ),
                        )
                        .collect();

                    if remaining_file_size > self.boot_sector.cluster_size {
                        remaining_file_size -= self.boot_sector.cluster_size;
                        current_cluster = next_cluster;
                    } else {
                        // add the remaining padding as 0 at the end of the cluster
                        self.storage_buffer[current_cluster]
                            .resize(self.boot_sector.cluster_size as usize, 0);
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

        Ok(())
    }

    fn list_files(&mut self, request: &ListRequest) -> Result<RootTable, Box<dyn Error>> {
        println!("{:?}", request.filters);
        println!("{:?}", request.sort);

        // filter away empty entries
        let mut file_entries: RootTable = self
            .root
            .iter()
            .filter(|&file_entry| !file_entry.name.is_empty())
            .cloned()
            .collect();

        // apply filters if any
        if !request.filters.is_empty() {
            file_entries.retain(|file_entry| {
                request.filters.iter().all(|filter| match filter {
                    FilterType::Name(name) => file_entry.name == *name,
                    FilterType::Extension(extension) => file_entry.extension == *extension,
                    FilterType::Files => file_entry.is_file(),
                    FilterType::Directories => !file_entry.is_file(),
                    FilterType::All => !file_entry.is_hidden(),
                    _ => true,
                })
            });
        } else {
            // as default, filter away hidden files
            file_entries.retain(|file_entry| !file_entry.is_hidden());
        }

        // apply sort if any
        if let Some(sort) = &request.sort {
            match &sort {
                SortType::NameAsc => {
                    file_entries.sort_by(|a, b| a.name.cmp(&b.name));
                }
                SortType::NameDesc => {
                    file_entries.sort_by(|a, b| b.name.cmp(&a.name));
                }
                SortType::DateAsc => {
                    file_entries.sort_by(|a, b| {
                        a.last_modification_datetime
                            .cmp(&b.last_modification_datetime)
                    });
                }
                SortType::DateDesc => {
                    file_entries.sort_by(|a, b| {
                        b.last_modification_datetime
                            .cmp(&a.last_modification_datetime)
                    });
                }
                SortType::SizeAsc => {
                    file_entries.sort_by(|a, b| a.size.cmp(&b.size));
                }
                SortType::SizeDesc => {
                    file_entries.sort_by(|a, b| b.size.cmp(&a.size));
                }
            }
        }

        Ok(file_entries)
    }

    fn rename_file(&mut self, request: &RenameRequest) -> Void {
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

        self.root[file_entry_index].name = request.new_name.to_owned();
        self.root[file_entry_index].extension = request.new_extension.to_owned();

        Ok(())
    }

    fn delete_file(&mut self, request: &DeleteRequest) -> Void {
        // check if the file exists in root
        if !self.root.iter().any(|file_entry| {
            file_entry.name == request.file_name && file_entry.extension == request.file_extension
        }) {
            return Err(Box::try_from(format!(
                "File {}.{} does not exist",
                request.file_name, request.file_extension
            ))
            .unwrap());
        }

        // delete the file in root and free the cluster chain in fat
        let file_entry_index = self
            .root
            .iter()
            .position(|file_entry| {
                file_entry.name == request.file_name
                    && file_entry.extension == request.file_extension
            })
            .unwrap_or_default();

        let file_entry = self.root[file_entry_index].clone();
        self.free_clusters_and_entry(&file_entry);

        Ok(())
    }

    fn get_file_content(&mut self, request: &CatRequest) -> Result<String, Box<dyn Error>> {
        // check if the file exists in root
        if !self.root.iter().any(|file_entry| {
            file_entry.name == request.file_name && file_entry.extension == request.file_extension
        }) {
            return Err(Box::try_from(format!(
                "File {}.{} does not exist",
                request.file_name, request.file_extension
            ))
            .unwrap());
        }

        // get the file entry
        let file_entry_index = self
            .root
            .iter()
            .position(|file_entry| {
                file_entry.name == request.file_name
                    && file_entry.extension == request.file_extension
            })
            .unwrap_or_default();

        // iterate through the cluster chain and read the file content from the storage buffer
        let mut content = String::new();
        let mut current_cluster = self.root[file_entry_index].first_cluster as usize;

        while self.fat[current_cluster] != FatValue::EndOfChain {
            content.push_str(&String::from_utf8_lossy(
                &self.storage_buffer[current_cluster][..self.boot_sector.cluster_size as usize],
            ));

            let next_cluster_index: u16 = self.fat[current_cluster].clone().into();
            current_cluster = next_cluster_index as usize;
        }

        // push the remaining content
        let remaining_content_size =
            self.root[file_entry_index].size as usize % self.boot_sector.cluster_size as usize;
        content.push_str(&String::from_utf8_lossy(
            &self.storage_buffer[current_cluster][..remaining_content_size],
        ));

        Ok(content)
    }

    fn copy_file(&mut self, request: &CopyRequest) -> Void {
        // check if the src file exists in root
        if !self.root.iter().any(|file_entry| {
            file_entry.name == request.src_name && file_entry.extension == request.src_extension
        }) {
            return Err(Box::try_from(format!(
                "File {}.{} does not exist",
                request.src_name, request.src_extension
            ))
            .unwrap());
        }

        // check if the dest file already exists in root
        if self.root.iter().any(|file_entry| {
            file_entry.name == request.dest_name && file_entry.extension == request.dest_extension
        }) {
            return Err(Box::try_from(format!(
                "File {}.{} already exists",
                request.dest_name, request.dest_extension
            ))
            .unwrap());
        }

        // check if there are empty entries in root
        if !self
            .root
            .iter()
            .any(|file_entry| file_entry.name.is_empty())
        {
            return Err(Box::try_from("No empty entries in root").unwrap());
        }

        // get the src file entry
        let src_file_entry_index = self
            .root
            .iter()
            .position(|file_entry| {
                file_entry.name == request.src_name && file_entry.extension == request.src_extension
            })
            .unwrap_or_default();

        // check if there is enough space in fat
        let src_file_entry = self.root[src_file_entry_index].clone();
        let src_file_size = src_file_entry.size as usize;
        let required_clusters = (src_file_size / self.boot_sector.cluster_size as usize) + 1;

        if self
            .fat
            .iter()
            .filter(|&fat_value| *fat_value == FatValue::Free)
            .count()
            < required_clusters
        {
            return Err(Box::try_from("Not enough space in fat").unwrap());
        }

        // create the dest file entry and register it in root
        let dest_file_first_cluster = self
            .fat
            .iter()
            .position(|fat_value| *fat_value == FatValue::Free)
            .unwrap() as u16;
        let dest_file_entry_index = self
            .root
            .iter()
            .position(|file_entry| file_entry.name.is_empty())
            .unwrap();

        let dest_file_entry = FileEntry::new(
            request.dest_name.to_owned(),
            request.dest_extension.to_owned(),
            src_file_entry.size,
            dest_file_first_cluster,
            FileEntryAttributes::File as u8,
            Utc::now(),
        );
        self.root[dest_file_entry_index] = dest_file_entry.clone();

        // iterate through the cluster chain and copy the file content from the storage buffer
        let mut current_src_cluster_index = src_file_entry.first_cluster as usize;
        let mut current_dest_cluster_index = dest_file_entry.first_cluster as usize;

        while self.fat[current_src_cluster_index] != FatValue::EndOfChain {
            self.storage_buffer[current_dest_cluster_index] =
                self.storage_buffer[current_src_cluster_index].clone();

            let next_dest_cluster_index: u16 = self
                .fat
                .iter()
                .enumerate()
                .position(|(cluster_index, fat_value)| {
                    *fat_value == FatValue::Free && cluster_index > current_dest_cluster_index
                })
                .unwrap() as u16;

            self.fat[current_dest_cluster_index] = FatValue::Data(next_dest_cluster_index);
            current_dest_cluster_index = next_dest_cluster_index as usize;

            let next_src_cluster_index: u16 = self.fat[current_src_cluster_index].clone().into();
            current_src_cluster_index = next_src_cluster_index as usize;
        }

        self.storage_buffer[current_dest_cluster_index] =
            self.storage_buffer[current_src_cluster_index].clone();
        self.fat[current_dest_cluster_index] = FatValue::EndOfChain;

        Ok(())
    }

    fn format_disk(&mut self, request: &FormatRequest) -> Void {
        // create a new in memory disk representation associated with the new fat type
        let mut boot_sector = self.get_boot_sector();
        let disk_size = boot_sector.cluster_size as u32 * boot_sector.cluster_count as u32;

        boot_sector.cluster_size = request.fat_type;
        boot_sector.cluster_count = (disk_size / (boot_sector.cluster_size as u32)) as u16;

        let mut new_disk_manager = DiskManager::new(CONFIG_ARC.clone(), boot_sector);

        // push sync the new disk representation to the storage
        new_disk_manager.push_sync();

        Ok(())
    }

    fn defragment_disk(&mut self) -> Void {
        // create a new temporary disk representation
        let mut new_disk_manager = DiskManager::new(CONFIG_ARC.clone(), self.get_boot_sector());

        // iterate over the root file entries
        for file_entry in self.root.clone().iter() {
            // skip empty file entries
            if file_entry.name.is_empty() {
                continue;
            }

            // get file content
            let cat_request =
                CatRequest::new(file_entry.name.clone(), file_entry.extension.clone());
            let file_content = self.get_file_content(&cat_request)?;

            // write the file content to the temp buffer file
            DiskManager::write_to_temp(file_content.as_str())?;

            // create a new file entry in the new disk representation
            let create_request = CreateRequest::new(
                file_entry.name.clone(),
                file_entry.extension.clone(),
                file_content.len() as u32,
                ContentType::Temp,
            );
            new_disk_manager.create_file(&create_request)?;

            // sync the datetime of the old file entry to the new file entry
            let new_file_entry = new_disk_manager
                .root
                .iter_mut()
                .find(|new_file_entry| {
                    new_file_entry.name == file_entry.name
                        && new_file_entry.extension == file_entry.extension
                })
                .unwrap();

            new_file_entry.last_modification_datetime = file_entry.last_modification_datetime;
        }

        // push sync
        new_disk_manager.push_sync();

        Ok(())
    }

    fn get_working_directory(&self) -> String {
        self.working_directory.name.clone()
    }

    fn get_boot_sector(&self) -> BootSector {
        self.boot_sector.clone()
    }

    fn get_free_space(&mut self) -> u64 {
        self.pull_sync();

        let free_clusters = self
            .fat
            .iter()
            .filter(|&fat_value| *fat_value == FatValue::Free)
            .count();

        (free_clusters * self.boot_sector.cluster_size as usize) as u64
    }

    fn get_total_space(&self) -> u64 {
        (self.fat.len() * self.boot_sector.cluster_size as usize) as u64
    }
}
