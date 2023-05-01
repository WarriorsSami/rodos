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
use crate::core::content_type::{ContentGenerator, ContentType};
use crate::core::filter_type::FilterType;
use crate::core::sort_type::SortType;
use crate::domain::boot_sector::BootSector;
use crate::domain::fat::FatValue;
use crate::domain::file_entry::{FileEntry, FileEntryAttributes, RootTable};
use crate::domain::i_disk_manager::IDiskManager;
use crate::infrastructure::disk_manager::DiskManager;
use crate::CONFIG_ARC;
use chrono::Utc;
use std::error::Error;

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
        if self
            .get_root_table_for_working_directory()
            .iter()
            .any(|file_entry| {
                file_entry.name == request.name && file_entry.extension == request.extension
            })
        {
            return Err(Box::try_from(format!(
                "File {}.{} already exists",
                request.name, request.extension
            ))
            .unwrap());
        }

        // check if there is enough space in root
        if self.working_directory.is_root()
            && self
                .root
                .iter()
                .all(|file_entry| !file_entry.name.is_empty())
        {
            return Err(Box::try_from("No space in root".to_string()).unwrap());
        }

        // check if there is enough space in fat
        let required_clusters =
            (request.size as f64 / self.boot_sector.cluster_size as f64).ceil() as usize;
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
        let first_cluster = self.get_next_free_cluster_index_gt(0).unwrap();

        // create file entry in root
        let file_entry = FileEntry::new(
            request.name.to_owned(),
            request.extension.to_owned(),
            request.size.to_owned(),
            first_cluster as u16,
            request.attributes,
            request.last_modification_datetime,
            Some(Box::new(self.working_directory.clone())),
            None,
        );

        // create the cluster chain in fat and write the file data to the storage buffer
        let mut current_cluster_index = first_cluster;
        let mut remaining_file_size = request.size as u16;
        let mut file_data = ContentGenerator::generate(request.content_type, request.size);

        while remaining_file_size > 0 {
            match self.get_next_free_cluster_index_gt(current_cluster_index) {
                Some(next_cluster_index) => {
                    self.fat[current_cluster_index] = FatValue::Data(next_cluster_index as u16);

                    self.storage_buffer[current_cluster_index] = file_data
                        .drain(
                            ..std::cmp::min(
                                self.boot_sector.cluster_size as usize,
                                remaining_file_size as usize,
                            ),
                        )
                        .collect();

                    if remaining_file_size > self.boot_sector.cluster_size {
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
                    self.free_clusters(&file_entry);
                    self.free_file_entry(&file_entry);
                    return Err(Box::try_from("No space in fat".to_string()).unwrap());
                }
            }
        }

        // update the root table
        self.append_to_root_table_of_working_dir(file_entry.clone())?;

        Ok(())
    }

    fn list_files(&mut self, request: &ListRequest) -> Result<RootTable, Box<dyn Error>> {
        // filter away empty entries
        let mut file_entries: RootTable = self
            .get_root_table_for_working_directory()
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
        match self.working_directory.is_root() {
            false => {
                // check if the old file exists in root table
                if !self
                    .get_root_table_for_working_directory()
                    .iter()
                    .any(|file_entry| {
                        file_entry.name == request.old_name
                            && file_entry.extension == request.old_extension
                    })
                {
                    let error_message = match request.old_extension.is_empty() {
                        true => format!("Directory {} does not exist", request.old_name),
                        false => format!(
                            "File {}.{} does not exist",
                            request.old_name, request.old_extension
                        ),
                    };

                    return Err(Box::try_from(error_message).unwrap());
                }

                // check if a file with the new name already exists in root table
                if self
                    .get_root_table_for_working_directory()
                    .iter()
                    .any(|file_entry| {
                        file_entry.name == request.new_name
                            && file_entry.extension == request.new_extension
                    })
                {
                    let error_message = match request.new_extension.is_empty() {
                        true => format!("Directory {} already exists", request.new_name),
                        false => format!(
                            "File {}.{} already exists",
                            request.new_name, request.new_extension
                        ),
                    };

                    return Err(Box::try_from(error_message).unwrap());
                }

                // get the index of the file entry in root table
                let root_table = self.get_root_table_for_working_directory();

                let file_entry_index = root_table
                    .iter()
                    .position(|file_entry| {
                        file_entry.name == request.old_name
                            && file_entry.extension == request.old_extension
                    })
                    .unwrap();

                // check if the file is read only
                if root_table[file_entry_index].is_read_only() {
                    let error_message = match request.old_extension.is_empty() {
                        true => format!("Directory {} is read only", request.old_name),
                        false => format!(
                            "File {}.{} is read only",
                            request.old_name, request.old_extension
                        ),
                    };

                    return Err(Box::try_from(error_message).unwrap());
                }

                // rename the file in root table
                root_table[file_entry_index].name = request.new_name.to_owned();
                root_table[file_entry_index].extension = request.new_extension.to_owned();
                root_table[file_entry_index].last_modification_datetime = Utc::now();

                self.sync_directory_root_table_to_storage(&self.working_directory.clone());
            }
            true => {
                // check if the old file exists in root table
                if !self.root.iter().any(|file_entry| {
                    file_entry.name == request.old_name
                        && file_entry.extension == request.old_extension
                }) {
                    let error_message = match request.old_extension.is_empty() {
                        true => format!("Directory {} does not exist", request.old_name),
                        false => format!(
                            "File {}.{} does not exist",
                            request.old_name, request.old_extension
                        ),
                    };

                    return Err(Box::try_from(error_message).unwrap());
                }

                // check if a file with the new name already exists in root table
                if self.root.iter().any(|file_entry| {
                    file_entry.name == request.new_name
                        && file_entry.extension == request.new_extension
                }) {
                    let error_message = match request.new_extension.is_empty() {
                        true => format!("Directory {} already exists", request.new_name),
                        false => format!(
                            "File {}.{} already exists",
                            request.new_name, request.new_extension
                        ),
                    };

                    return Err(Box::try_from(error_message).unwrap());
                }

                // get the index of the file entry in root table
                let file_entry_index = self
                    .root
                    .iter()
                    .position(|file_entry| {
                        file_entry.name == request.old_name
                            && file_entry.extension == request.old_extension
                    })
                    .unwrap();

                // check if the file is read only
                if self.root[file_entry_index].is_read_only() {
                    let error_message = match request.old_extension.is_empty() {
                        true => format!("Directory {} is read only", request.old_name),
                        false => format!(
                            "File {}.{} is read only",
                            request.old_name, request.old_extension
                        ),
                    };

                    return Err(Box::try_from(error_message).unwrap());
                }

                // rename the file in root table
                self.root[file_entry_index].name = request.new_name.to_owned();
                self.root[file_entry_index].extension = request.new_extension.to_owned();
                self.root[file_entry_index].last_modification_datetime = Utc::now();
            }
        }

        Ok(())
    }

    fn delete_file(&mut self, request: &DeleteRequest) -> Void {
        // check if the file exists in root
        if !self
            .get_root_table_for_working_directory()
            .iter()
            .any(|file_entry| {
                file_entry.name == request.file_name
                    && file_entry.extension == request.file_extension
                    && file_entry.is_file()
            })
        {
            return Err(Box::try_from(format!(
                "File {}.{} does not exist",
                request.file_name, request.file_extension
            ))
            .unwrap());
        }

        // get the file entry index in root
        let file_entry = self
            .get_root_table_for_working_directory()
            .iter()
            .find(|file_entry| {
                file_entry.name == request.file_name
                    && file_entry.extension == request.file_extension
                    && file_entry.is_file()
            })
            .cloned()
            .unwrap();

        // check if it is read only
        if file_entry.is_read_only() {
            return Err(Box::try_from(format!(
                "File {}.{} is read only",
                request.file_name, request.file_extension
            ))
            .unwrap());
        }

        // delete the file in root and free the cluster chain in fat
        self.free_clusters(&file_entry);
        self.free_file_entry(&file_entry);

        Ok(())
    }

    fn get_file_content(&mut self, request: &CatRequest) -> Result<String, Box<dyn Error>> {
        // check if the file exists in the working directory
        if !self
            .get_root_table_for_working_directory()
            .iter()
            .any(|file_entry| {
                file_entry.name == request.file_name
                    && file_entry.extension == request.file_extension
                    && file_entry.is_file()
            })
        {
            return Err(Box::try_from(format!(
                "File {}.{} does not exist",
                request.file_name, request.file_extension
            ))
            .unwrap());
        }

        // get the file entry
        let file_entry = self
            .get_root_table_for_working_directory()
            .iter()
            .find(|&file_entry| {
                file_entry.name == request.file_name
                    && file_entry.extension == request.file_extension
                    && file_entry.is_file()
            })
            .cloned()
            .unwrap();

        // iterate through the cluster chain and read the file content from the storage buffer
        let mut content = String::new();
        let mut current_cluster = file_entry.first_cluster as usize;

        while self.fat[current_cluster] != FatValue::EndOfChain {
            content.push_str(&String::from_utf8_lossy(
                &self.storage_buffer[current_cluster][..self.boot_sector.cluster_size as usize],
            ));

            let next_cluster_index: u16 = self.fat[current_cluster].clone().into();
            current_cluster = next_cluster_index as usize;
        }

        // push the remaining content
        let remaining_content_size =
            file_entry.size as usize % self.boot_sector.cluster_size as usize;
        content.push_str(&String::from_utf8_lossy(
            &self.storage_buffer[current_cluster][..remaining_content_size],
        ));

        Ok(content)
    }

    fn copy_file(&mut self, request: &CopyRequest) -> Void {
        // check if the src file exists in root
        if !self
            .get_root_table_for_working_directory()
            .iter()
            .any(|file_entry| {
                file_entry.name == request.src_name && file_entry.extension == request.src_extension
            })
        {
            return Err(Box::try_from(format!(
                "File {}.{} does not exist",
                request.src_name, request.src_extension
            ))
            .unwrap());
        }

        // check if the dest file already exists in root
        if self
            .get_root_table_for_working_directory()
            .iter()
            .any(|file_entry| {
                file_entry.name == request.dest_name
                    && file_entry.extension == request.dest_extension
            })
        {
            return Err(Box::try_from(format!(
                "File {}.{} already exists",
                request.dest_name, request.dest_extension
            ))
            .unwrap());
        }

        // check if there are empty entries in root when working directory is root
        if self.working_directory.is_root()
            && !self
                .root
                .iter()
                .any(|file_entry| file_entry.name.is_empty())
        {
            return Err(Box::try_from("No empty entries in root").unwrap());
        }

        // get the src file entry
        let src_file_entry = self
            .get_root_table_for_working_directory()
            .iter()
            .find(|file_entry| {
                file_entry.name == request.src_name && file_entry.extension == request.src_extension
            })
            .cloned()
            .unwrap();

        // check if there is enough space in fat
        let required_clusters =
            (src_file_entry.size as f32 / self.boot_sector.cluster_size as f32).ceil() as usize;

        if self
            .fat
            .iter()
            .filter(|&fat_value| *fat_value == FatValue::Free)
            .count()
            < required_clusters
        {
            return Err(Box::try_from("Not enough space in fat").unwrap());
        }

        // create the dest file entry
        let dest_file_first_cluster = self.get_next_free_cluster_index_gt(0).unwrap() as u16;
        let dest_file_entry = FileEntry::new(
            request.dest_name.to_owned(),
            request.dest_extension.to_owned(),
            src_file_entry.size,
            dest_file_first_cluster,
            src_file_entry.attributes,
            Utc::now(),
            src_file_entry.parent_entry.clone(),
            src_file_entry.children_entries.clone(),
        );

        // iterate through the cluster chain and copy the file content from the storage buffer
        let mut current_src_cluster_index = src_file_entry.first_cluster as usize;
        let mut current_dest_cluster_index = dest_file_entry.first_cluster as usize;

        while self.fat[current_src_cluster_index] != FatValue::EndOfChain {
            self.storage_buffer[current_dest_cluster_index] =
                self.storage_buffer[current_src_cluster_index].clone();

            let next_dest_cluster_index: u16 = self
                .get_next_free_cluster_index_gt(current_dest_cluster_index)
                .unwrap() as u16;

            self.fat[current_dest_cluster_index] = FatValue::Data(next_dest_cluster_index);
            current_dest_cluster_index = next_dest_cluster_index as usize;

            let next_src_cluster_index: u16 = self.fat[current_src_cluster_index].clone().into();
            current_src_cluster_index = next_src_cluster_index as usize;
        }

        self.storage_buffer[current_dest_cluster_index] =
            self.storage_buffer[current_src_cluster_index].clone();
        self.fat[current_dest_cluster_index] = FatValue::EndOfChain;

        // append the dest file entry to the root table of the working directory
        self.append_to_root_table_of_working_dir(dest_file_entry)?;

        Ok(())
    }

    fn set_attributes(&mut self, request: &SetAttributesRequest) -> Void {
        // check if the file exists
        if !self
            .get_root_table_for_working_directory()
            .iter()
            .any(|file_entry| {
                file_entry.name == request.name && file_entry.extension == request.extension
            })
        {
            return Err(Box::try_from(format!(
                "File {}.{} does not exist",
                request.name, request.extension
            ))
            .unwrap());
        }

        if self.working_directory.is_root() {
            // get the file entry
            let file_entry_index = self
                .root
                .iter()
                .position(|file_entry| {
                    file_entry.name == request.name && file_entry.extension == request.extension
                })
                .unwrap_or_default();

            // set the attributes
            self.root[file_entry_index].apply_attributes(&request.attributes);
            self.root[file_entry_index].last_modification_datetime = Utc::now();
        } else {
            let file_entry = self
                .get_root_table_for_working_directory()
                .iter_mut()
                .find(|file_entry| {
                    file_entry.name == request.name && file_entry.extension == request.extension
                })
                .unwrap();

            // set the attributes
            file_entry.apply_attributes(&request.attributes);
            file_entry.last_modification_datetime = Utc::now();

            // persist the file entry modifications into the storage
            self.sync_directory_root_table_to_storage(&self.working_directory.clone());
        }

        Ok(())
    }

    fn format_disk(&mut self, request: &FormatRequest) -> Void {
        // create a new in memory disk representation associated with the new fat type
        let mut boot_sector = self.get_boot_sector().clone();
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
        let mut new_disk_manager =
            DiskManager::new(CONFIG_ARC.clone(), self.get_boot_sector().clone());

        // change the working directory to be the root
        while !self.working_directory.is_root() {
            let cd_request = ChangeDirectoryRequest::new("..".to_string());
            self.change_working_directory(&cd_request)?;
        }

        // iterate over the root file entries
        for file_entry in self.root.clone().iter() {
            // skip empty file entries
            if file_entry.name.is_empty() {
                continue;
            }

            // create a new file entry in the new disk representation
            match file_entry.is_file() {
                true => {
                    // get file content
                    let cat_request =
                        CatRequest::new(file_entry.name.clone(), file_entry.extension.clone());
                    let file_content = self.get_file_content(&cat_request)?;

                    // write the file content to the temp buffer file
                    DiskManager::write_to_temp_buffer(file_content.as_str())?;

                    // create the file entry
                    let create_request = CreateRequest::new(
                        file_entry.name.clone(),
                        file_entry.extension.clone(),
                        file_content.len() as u32,
                        file_entry.attributes,
                        file_entry.last_modification_datetime,
                        ContentType::Temp,
                    );
                    new_disk_manager.create_file(&create_request)?;
                }
                false => {
                    // create the directory entry
                    let make_directory_request = MakeDirectoryRequest::new(
                        file_entry.name.clone(),
                        file_entry.attributes,
                        file_entry.last_modification_datetime,
                    );
                    new_disk_manager.make_directory(&make_directory_request)?;

                    // iterate over the directory's root table and recreate the dir tree in the new disk representation
                    self.inflate_directory_tree(&mut new_disk_manager, file_entry)?;
                }
            }
        }

        // push sync
        new_disk_manager.push_sync();

        Ok(())
    }

    fn make_directory(&mut self, request: &MakeDirectoryRequest) -> Void {
        // check if the directory name already exists
        if self
            .get_root_table_for_working_directory()
            .iter()
            .any(|file_entry| file_entry.name == request.name && !file_entry.is_file())
        {
            return Err(
                Box::try_from(format!("Directory {} already exists", request.name)).unwrap(),
            );
        }

        // check if there is enough space in fat
        let free_clusters = self
            .fat
            .iter()
            .filter(|&fat_value| *fat_value == FatValue::Free)
            .count();

        if free_clusters == 0 {
            return Err(Box::try_from("Not enough space in FAT").unwrap());
        }

        // create the directory file entry and attach the two special dir entries: `.` and `..`
        let first_cluster_index = self
            .fat
            .iter()
            .position(|fat_value| *fat_value == FatValue::Free)
            .unwrap() as u16;

        let mut dir_file_entry = FileEntry::new(
            request.name.clone(),
            "".to_string(),
            (self.boot_sector.root_entry_cell_size * 2) as u32,
            first_cluster_index,
            request.attributes,
            request.last_modification_datetime,
            Some(Box::new(self.working_directory.clone())),
            Some(Vec::new()),
        );

        let mut dot_dir_entry = dir_file_entry.clone();
        dot_dir_entry.name = ".".to_string();
        dot_dir_entry.attributes = FileEntryAttributes::combine(&[
            FileEntryAttributes::Directory,
            FileEntryAttributes::ReadOnly,
            FileEntryAttributes::Hidden,
        ]);

        let mut double_dot_dir_entry = self.working_directory.clone();
        double_dot_dir_entry.name = "..".to_string();
        double_dot_dir_entry.extension = "".to_string();
        double_dot_dir_entry.attributes = FileEntryAttributes::combine(&[
            FileEntryAttributes::Directory,
            FileEntryAttributes::ReadOnly,
            FileEntryAttributes::Hidden,
        ]);

        dir_file_entry.children_entries = Some(vec![dot_dir_entry, double_dot_dir_entry]);

        // update the fat and the storage
        let mut current_cluster_index = first_cluster_index as usize;
        let mut remaining_file_size = dir_file_entry.size as u16;
        let mut file_data = DiskManager::serialize_directory_root_table(&dir_file_entry);

        while remaining_file_size > 0 {
            match self.get_next_free_cluster_index_gt(current_cluster_index) {
                Some(next_cluster_index) => {
                    self.fat[current_cluster_index] = FatValue::Data(next_cluster_index as u16);

                    self.storage_buffer[current_cluster_index] = file_data
                        .drain(
                            ..std::cmp::min(
                                self.boot_sector.cluster_size as usize,
                                remaining_file_size as usize,
                            ),
                        )
                        .collect();

                    if remaining_file_size > self.boot_sector.cluster_size {
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
                    self.free_clusters(&dir_file_entry);
                    self.free_file_entry(&dir_file_entry);
                    return Err(Box::try_from("No space in fat".to_string()).unwrap());
                }
            }
        }

        // update the root table
        self.append_to_root_table_of_working_dir(dir_file_entry.clone())?;

        Ok(())
    }

    fn change_working_directory(&mut self, request: &ChangeDirectoryRequest) -> Void {
        // check if the directory exists
        if !self
            .get_root_table_for_working_directory()
            .iter()
            .any(|file_entry| file_entry.name == request.directory_name && !file_entry.is_file())
        {
            return Err(Box::try_from(format!(
                "Directory {} does not exist",
                request.directory_name
            ))
            .unwrap());
        }

        if request.directory_name == "." {
            return Ok(());
        }

        if request.directory_name == ".." {
            let parent = self.working_directory.parent_entry.clone().unwrap();
            self.working_directory = *parent;
            return Ok(());
        }

        // change the working directory
        self.working_directory = self
            .get_root_table_for_working_directory()
            .iter()
            .find(|file_entry| file_entry.name == request.directory_name && !file_entry.is_file())
            .unwrap()
            .clone();

        Ok(())
    }

    fn get_working_directory_full_path(&self) -> String {
        // construct the whole path from the root to the working directory
        let mut dirs: Vec<&str> = Vec::new();
        dirs.push(&self.working_directory.name);

        let mut current_dir = &self.working_directory;
        while let Some(parent_dir) = current_dir.parent_entry.as_ref() {
            current_dir = parent_dir;
            dirs.push(&current_dir.name);
        }

        if self.working_directory.parent_entry.is_some() {
            dirs.pop();
            dirs.push("");
        }

        dirs.reverse();
        dirs.join("/")
    }

    fn get_boot_sector(&self) -> &BootSector {
        &self.boot_sector
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
