use crate::domain::disk_manager::ByteArray;
use std::ops::BitOr;

pub(crate) enum FileEntryAttributes {
    ReadOnly = 0x01,
    Hidden = 0x02,
    File = 0x04,
}

impl BitOr for FileEntryAttributes {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FileEntry {
    pub(crate) name: String,
    pub(crate) extension: String,
    pub(crate) size: u32,
    pub(crate) first_cluster: u32,
    pub(crate) attributes: u8,
}

impl From<ByteArray> for FileEntry {
    fn from(value: ByteArray) -> Self {
        let mut name = String::new();
        let mut extension = String::new();

        (0..8).for_each(|i| name.push(value[i] as char));
        (8..11).for_each(|i| extension.push(value[i] as char));

        let size = u16::from_be_bytes([value[12], value[13]]);
        let first_cluster = u16::from_be_bytes([value[14], value[15]]);
        let attributes = value[16];

        Self {
            name,
            extension,
            size: size as u32,
            first_cluster: first_cluster as u32,
            attributes,
        }
    }
}

impl Into<ByteArray> for FileEntry {
    fn into(self) -> ByteArray {
        let mut result = Vec::new();
        result.resize(16, 0);

        let name = self.name.as_bytes();
        let extension = self.extension.as_bytes();

        name.iter()
            .enumerate()
            .for_each(|(index, &value)| result[index] = value);
        extension
            .iter()
            .enumerate()
            .for_each(|(index, &value)| result[index + 8] = value);

        let size = self.size.to_be_bytes();
        let first_cluster = self.first_cluster.to_be_bytes();

        result[11] = size[0];
        result[12] = size[1];
        result[13] = first_cluster[0];
        result[14] = first_cluster[1];
        result[15] = self.attributes;

        result
    }
}

pub(crate) type RootTable = Vec<FileEntry>;
