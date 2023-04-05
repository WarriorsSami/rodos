#[derive(Debug, Clone, Default)]
pub(crate) struct FileEntry {
    pub(crate) name: String,
    pub(crate) extension: String,
    pub(crate) size: u32,
    pub(crate) first_cluster: u32,
    pub(crate) attributes: u8,
}

impl From<Vec<u8>> for FileEntry {
    fn from(value: Vec<u8>) -> Self {
        let mut name = String::new();
        let mut extension = String::new();

        (0..8).for_each(|i| name.push(value[i] as char));
        (8..11).for_each(|i| extension.push(value[i] as char));

        let size = u16::from_le_bytes([value[13], value[12]]);
        let first_cluster = u16::from_le_bytes([value[15], value[14]]);
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

impl Into<Vec<u8>> for FileEntry {
    fn into(self) -> Vec<u8> {
        let mut result = Vec::new();
        result.resize(32, 0);

        let name = self.name.as_bytes();
        let extension = self.extension.as_bytes();

        (0..8).for_each(|i| result[i] = name[i]);
        (8..11).for_each(|i| result[i] = extension[i - 8]);

        let size = self.size.to_le_bytes();
        let first_cluster = self.first_cluster.to_le_bytes();

        result[12] = size[0];
        result[13] = size[1];
        result[14] = first_cluster[0];
        result[15] = first_cluster[1];
        result[16] = self.attributes;

        result
    }
}

pub(crate) type RootTable = Vec<FileEntry>;
