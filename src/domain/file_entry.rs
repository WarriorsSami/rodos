use crate::infrastructure::disk_manager::ByteArray;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use std::fmt::Display;
use std::ops::BitOr;

#[derive(Debug, Clone)]
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
    pub(crate) first_cluster: u16,
    pub(crate) attributes: u8,
    pub(crate) last_modification_datetime: DateTime<Utc>,
}

impl FileEntry {
    pub(crate) fn new(
        name: String,
        extension: String,
        size: u32,
        first_cluster: u16,
        attributes: u8,
        last_modification_datetime: DateTime<Utc>,
    ) -> Self {
        Self {
            name,
            extension,
            size,
            first_cluster,
            attributes,
            last_modification_datetime,
        }
    }

    pub(crate) fn root() -> Self {
        Self {
            name: "/".to_string(),
            extension: "".to_string(),
            size: 0,
            first_cluster: 0,
            attributes: 0,
            last_modification_datetime: Utc::now(),
        }
    }

    pub(crate) fn get_attributes_as_string(&self) -> String {
        let mut result = String::new();

        if self.attributes & FileEntryAttributes::File as u8 != 0 {
            result.push('f');
        } else {
            result.push('d');
        }

        if self.attributes & FileEntryAttributes::ReadOnly as u8 != 0 {
            result.push('r');
        } else {
            result.push('w');
        }

        if self.attributes & FileEntryAttributes::Hidden as u8 != 0 {
            result.push('h');
        } else {
            result.push('v');
        }

        result
    }

    fn convert_u16_tuple_to_date_time(value: (u16, u16)) -> DateTime<Utc> {
        let year = (value.0 >> 9) + 1980;
        let month = (value.0 >> 5) & 0x0F;
        let day = value.0 & 0x1F;
        let hour = (value.1 >> 11) & 0x1F;
        let minute = (value.1 >> 5) & 0x3F;
        let second = (value.1 & 0x1F) * 2;

        Utc.with_ymd_and_hms(
            year as i32,
            month as u32,
            day as u32,
            hour as u32,
            minute as u32,
            second as u32,
        )
        .unwrap()
    }

    pub(crate) fn is_file(&self) -> bool {
        self.attributes & FileEntryAttributes::File as u8 != 0
    }

    pub(crate) fn is_hidden(&self) -> bool {
        self.attributes & FileEntryAttributes::Hidden as u8 != 0
    }

    pub(crate) fn is_read_only(&self) -> bool {
        self.attributes & FileEntryAttributes::ReadOnly as u8 != 0
    }
}

impl Display for FileEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {}.{} {} ({} B)",
            self.get_attributes_as_string(),
            self.name,
            self.extension,
            self.last_modification_datetime,
            self.size
        )
    }
}

impl From<ByteArray> for FileEntry {
    fn from(value: ByteArray) -> Self {
        let mut name = String::new();
        let mut extension = String::new();

        (0..8).for_each(|i| {
            if value[i] != 0x00 {
                name.push(value[i] as char);
            }
        });
        (8..11).for_each(|i| {
            if value[i] != 0x00 {
                extension.push(value[i] as char);
            }
        });

        let size = u32::from_be_bytes([value[11], value[12], value[13], value[14]]);
        let first_cluster = u16::from_be_bytes([value[15], value[16]]);
        let attributes = value[17];

        let time = u16::from_be_bytes([value[18], value[19]]);
        let date = u16::from_be_bytes([value[20], value[21]]);

        let updated_datetime = FileEntry::convert_u16_tuple_to_date_time((date, time));

        Self {
            name,
            extension,
            size,
            first_cluster,
            attributes,
            last_modification_datetime: updated_datetime,
        }
    }
}

impl Into<ByteArray> for FileEntry {
    fn into(self) -> ByteArray {
        let mut result = Vec::new();
        result.resize(32, 0);

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
        result[13] = size[2];
        result[14] = size[3];

        result[15] = first_cluster[0];
        result[16] = first_cluster[1];

        result[17] = self.attributes;

        let time = self.last_modification_datetime.time();
        let date = self.last_modification_datetime.date_naive();

        let time = (time.hour() << 11) | (time.minute() << 5) | (time.second() / 2);
        let date = ((date.year() - 1980) << 9) as u32 | date.month() << 5 | date.day();

        let time = (time as u16).to_be_bytes();
        let date = (date as u16).to_be_bytes();

        result[18] = time[0];
        result[19] = time[1];
        result[20] = date[0];
        result[21] = date[1];

        result
    }
}

pub(crate) type RootTable = Vec<FileEntry>;
