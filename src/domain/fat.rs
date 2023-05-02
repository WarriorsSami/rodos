/// FAT16/32 possible values:
/// - 0x0000: Free
/// - 0x0001: Reserved
/// - 0x0002: Bad
/// - 0xFFFF: End of chain
/// - 0x0003-0xFFFE: Data
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FatValue {
    Free,
    Reserved,
    EndOfChain,
    Data(u16),
    Bad,
}

/// Serializes a `FatValue` into a `u16`.
impl From<FatValue> for u16 {
    fn from(value: FatValue) -> Self {
        match value {
            FatValue::Free => 0x0000,
            FatValue::Reserved => 0x0001,
            FatValue::Bad => 0x0002,
            FatValue::Data(value) => value,
            FatValue::EndOfChain => 0xFFFF,
        }
    }
}

/// Deserializes a `u16` into a `FatValue`.
impl From<u16> for FatValue {
    fn from(value: u16) -> Self {
        match value {
            0x0000 => FatValue::Free,
            0x0001 => FatValue::Reserved,
            0x0002 => FatValue::Bad,
            0xFFFF => FatValue::EndOfChain,
            value => FatValue::Data(value),
        }
    }
}

/// A FAT table.
pub(crate) type FatTable = Vec<FatValue>;
