use crate::infrastructure::disk_manager::ByteArray;

#[derive(Debug, Clone)]
pub(crate) struct BootSector {
    pub(crate) cluster_size: u16,
    pub(crate) cluster_count: u16,
    pub(crate) root_entry_cell_size: u16,
    pub(crate) root_entry_count: u16,
    pub(crate) fat_cell_size: u16,
    pub(crate) clusters_per_boot_sector: u16,
}

impl Default for BootSector {
    fn default() -> Self {
        Self {
            cluster_size: 16,
            cluster_count: 8192,
            root_entry_cell_size: 32,
            root_entry_count: 64,
            fat_cell_size: 2,
            clusters_per_boot_sector: 1,
        }
    }
}

impl From<ByteArray> for BootSector {
    fn from(value: ByteArray) -> Self {
        // cluster_size
        let cluster_size = u16::from_be_bytes([value[0], value[1]]);

        // cluster_count
        let cluster_count = u16::from_be_bytes([value[2], value[3]]);

        // root_entry_cell_size
        let root_entry_cell_size = u16::from_be_bytes([value[4], value[5]]);

        // root_entry_count
        let root_entry_count = u16::from_be_bytes([value[6], value[7]]);

        // fat_cell_size
        let fat_cell_size = u16::from_be_bytes([value[8], value[9]]);

        // clusters_per_boot_sector
        let clusters_per_boot_sector = u16::from_be_bytes([value[10], value[11]]);

        Self {
            cluster_size,
            cluster_count,
            root_entry_cell_size,
            root_entry_count,
            fat_cell_size,
            clusters_per_boot_sector,
        }
    }
}

impl Into<ByteArray> for BootSector {
    fn into(self) -> ByteArray {
        let mut result = Vec::new();

        result.resize(self.cluster_size as usize, 0);

        // cluster_size
        let cluster_size = self.cluster_size.to_be_bytes();
        result[0] = cluster_size[0];
        result[1] = cluster_size[1];

        // cluster_count
        let cluster_count = self.cluster_count.to_be_bytes();
        result[2] = cluster_count[0];
        result[3] = cluster_count[1];

        // root_entry_cell_size
        let root_entry_cell_size = self.root_entry_cell_size.to_be_bytes();
        result[4] = root_entry_cell_size[0];
        result[5] = root_entry_cell_size[1];

        // root_entry_count
        let root_entry_count = self.root_entry_count.to_be_bytes();
        result[6] = root_entry_count[0];
        result[7] = root_entry_count[1];

        // fat_cell_size
        let fat_cell_size = self.fat_cell_size.to_be_bytes();
        result[8] = fat_cell_size[0];
        result[9] = fat_cell_size[1];

        // clusters_per_boot_sector
        let clusters_per_boot_sector = self.clusters_per_boot_sector.to_be_bytes();
        result[10] = clusters_per_boot_sector[0];
        result[11] = clusters_per_boot_sector[1];

        result
    }
}
