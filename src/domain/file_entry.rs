use crate::infrastructure::ByteArray;
use chrono::{DateTime, Datelike, LocalResult, TimeZone, Timelike, Utc};
use std::fmt::Display;
use std::ops::BitOr;
use std::str::FromStr;

/// An enum representing the bit position as a bitmask for each type of file entry attribute:
/// - `Mode` (read-only or read-write): `bit 0`
/// - `Visibility` (hidden or visible): `bit 1`
/// - `Type` (file or directory): `bit 2`
#[derive(Debug, Clone, Copy)]
pub(crate) enum FileEntryAttributesFlags {
    Mode = 0x01,
    Visibility = 0x02,
    Type = 0x04,
}

/// An enum representing the value of each type of file entry attribute:
/// - `ReadOnly`: `set bit`
/// - `ReadWrite`: `unset bit`
/// - `Hidden`: `set bit`
/// - `Visible`: `unset bit`
/// - `File`: `set bit`
/// - `Directory`: `unset bit`
#[derive(Debug, Clone, Copy)]
pub(crate) enum FileEntryAttributes {
    /// Read-only file: `set bit`
    ReadOnly,
    /// Read-write file: `unset bit`
    ReadWrite,
    /// Hidden file: `set bit`
    Hidden,
    /// Visible file: `unset bit`
    Visible,
    /// File: `set bit`
    File,
    /// Directory: `unset bit`
    Directory,
}

impl FileEntryAttributes {
    /// Combines multiple `FileEntryAttributes` into a single `u8` value using the `BitOr` operator.
    pub(crate) fn combine(attributes: &[FileEntryAttributes]) -> u8 {
        attributes.iter().fold(0, |acc, x| {
            let x: u8 = (*x).into();
            acc | x
        })
    }
}

/// Serializes a `FileEntryAttributes` into a `u8` value.
impl Into<u8> for FileEntryAttributes {
    fn into(self) -> u8 {
        match self {
            FileEntryAttributes::ReadOnly => 0x01,
            FileEntryAttributes::ReadWrite => 0x00,
            FileEntryAttributes::Hidden => 0x02,
            FileEntryAttributes::Visible => 0x00,
            FileEntryAttributes::File => 0x04,
            FileEntryAttributes::Directory => 0x00,
        }
    }
}

/// Performs a bitwise OR operation between two `FileEntryAttributes` values.
impl BitOr for FileEntryAttributes {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        let lhs: u8 = self.into();
        let rhs: u8 = rhs.into();

        lhs | rhs
    }
}

/// Maps a `FileEntryAttributes` to the corresponding string flag:
/// - `ReadOnly`: `-w`
/// - `ReadWrite`: `+w`
/// - `Hidden`: `+h`
/// - `Visible`: `-h`
impl FromStr for FileEntryAttributes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-w" => Ok(FileEntryAttributes::ReadOnly),
            "+w" => Ok(FileEntryAttributes::ReadWrite),
            "-h" => Ok(FileEntryAttributes::Visible),
            "+h" => Ok(FileEntryAttributes::Hidden),
            _ => Err(format!("Invalid attribute {}", s)),
        }
    }
}

/// A file entry struct containing the necessary metadata to represent a file or directory:
/// - `name`: the name of the file or directory (8 bytes)
/// - `extension`: the extension of the file or directory (3 bytes)
/// - `size`: the size of the file or directory in bytes (4 bytes)
/// - `first_cluster`: the index of the allocation chain's head of the file or directory (2 bytes)
/// - `attributes`: the attributes of the file or directory (1 byte)
/// - `last_modification_datetime`: the last modification date and time of the file or directory (4 bytes)
/// where:
///     - `year`: 7 bits (0-127) since 1980
///     - `month`: 4 bits (1-12)
///     - `day`: 5 bits (1-31)
///     - `hour`: 5 bits (0-23)
///     - `minute`: 6 bits (0-59)
///     - `second`: 5 bits (0-59)
/// - `parent_entry`: the parent directory of the file or directory (none if root)
/// - `children_entries`: the children files or directories of the file or directory (none if file)
#[derive(Debug, Clone, Default)]
pub(crate) struct FileEntry {
    pub(crate) name: String,
    pub(crate) extension: String,
    pub(crate) size: u32,
    pub(crate) first_cluster: u16,
    pub(crate) attributes: u8,
    pub(crate) last_modification_datetime: DateTime<Utc>,
    pub(crate) parent_entry: Option<Box<FileEntry>>,
    pub(crate) children_entries: Option<Vec<FileEntry>>,
}

impl FileEntry {
    pub(crate) fn new(
        name: String,
        extension: String,
        size: u32,
        first_cluster: u16,
        attributes: u8,
        last_modification_datetime: DateTime<Utc>,
        parent_entry: Option<Box<FileEntry>>,
        children_entries: Option<Vec<FileEntry>>,
    ) -> Self {
        Self {
            name,
            extension,
            size,
            first_cluster,
            attributes,
            last_modification_datetime,
            parent_entry,
            children_entries,
        }
    }

    pub(crate) fn root() -> Self {
        Self {
            name: "/".to_string(),
            extension: "".to_string(),
            size: 0,
            first_cluster: 0,
            attributes: FileEntryAttributes::ReadOnly as u8,
            last_modification_datetime: Utc::now(),
            parent_entry: None,
            children_entries: Some(Vec::new()),
        }
    }

    pub(crate) fn is_root(&self) -> bool {
        self.name == "/"
    }

    /// Applies the given attributes against the current attributes of the file entry.
    pub(crate) fn apply_attributes(&mut self, attributes: &Vec<FileEntryAttributes>) {
        for attribute in attributes {
            match attribute {
                FileEntryAttributes::ReadOnly => {
                    let mode_bit = self.attributes & FileEntryAttributesFlags::Mode as u8;

                    if mode_bit == 0 {
                        self.attributes |= FileEntryAttributesFlags::Mode as u8;
                    }
                }
                FileEntryAttributes::ReadWrite => {
                    let mode_bit = self.attributes & FileEntryAttributesFlags::Mode as u8;

                    if mode_bit != 0 {
                        self.attributes &= !(FileEntryAttributesFlags::Mode as u8);
                    }
                }
                FileEntryAttributes::Hidden => {
                    let visibility_bit =
                        self.attributes & FileEntryAttributesFlags::Visibility as u8;

                    if visibility_bit == 0 {
                        self.attributes |= FileEntryAttributesFlags::Visibility as u8;
                    }
                }
                FileEntryAttributes::Visible => {
                    let visibility_bit =
                        self.attributes & FileEntryAttributesFlags::Visibility as u8;

                    if visibility_bit != 0 {
                        self.attributes &= !(FileEntryAttributesFlags::Visibility as u8);
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn get_attributes_as_string(&self) -> String {
        let mut result = String::new();

        if self.attributes & FileEntryAttributesFlags::Type as u8 != 0 {
            result.push('f');
        } else {
            result.push('d');
        }

        if self.attributes & FileEntryAttributesFlags::Mode as u8 != 0 {
            result.push('r');
        } else {
            result.push('w');
        }

        if self.attributes & FileEntryAttributesFlags::Visibility as u8 != 0 {
            result.push('h');
        } else {
            result.push('v');
        }

        result
    }

    fn convert_u16_tuple_to_date_time(value: (u16, u16)) -> LocalResult<DateTime<Utc>> {
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
    }

    pub(crate) fn is_file(&self) -> bool {
        self.attributes & FileEntryAttributesFlags::Type as u8 != 0
    }

    pub(crate) fn is_hidden(&self) -> bool {
        self.attributes & FileEntryAttributesFlags::Visibility as u8 != 0
    }

    pub(crate) fn is_read_only(&self) -> bool {
        self.attributes & FileEntryAttributesFlags::Mode as u8 != 0
    }
}

impl Display for FileEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.is_file() {
            true => write!(
                f,
                "{} - {}.{} {} ({} B)",
                self.get_attributes_as_string(),
                self.name,
                self.extension,
                self.last_modification_datetime,
                self.size
            ),
            false => write!(
                f,
                "{} - {} {} ({} B)",
                self.get_attributes_as_string(),
                self.name,
                self.last_modification_datetime,
                self.size
            ),
        }
    }
}

/// Deserializes a byte array into a file entry.
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

        let last_modification_datetime =
            match FileEntry::convert_u16_tuple_to_date_time((date, time)) {
                LocalResult::Single(value) => value,
                _ => {
                    panic!("Invalid date and time: {:?}", value);
                }
            };

        Self {
            name,
            extension,
            size,
            first_cluster,
            attributes,
            last_modification_datetime,
            parent_entry: None,
            children_entries: None,
        }
    }
}

/// Serializes a file entry into a byte array.
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

/// A root table is a list of file entries.
pub(crate) type RootTable = Vec<FileEntry>;
