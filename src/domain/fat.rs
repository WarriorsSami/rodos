#[derive(Debug, Clone)]
pub(crate) enum FatValue {
    Free,
    Reserved,
    EndOfChain,
    Data(u32),
    Bad,
}

impl From<FatValue> for u16 {
    fn from(value: FatValue) -> Self {
        match value {
            FatValue::Free => 0x0000,
            FatValue::Reserved => 0x0001,
            FatValue::Bad => 0x0002,
            FatValue::Data(value) => value as u16,
            FatValue::EndOfChain => 0xFFFF,
        }
    }
}

impl From<u16> for FatValue {
    fn from(value: u16) -> Self {
        match value {
            0x0000 => FatValue::Free,
            0x0001 => FatValue::Reserved,
            0x0002 => FatValue::Bad,
            0xFFFF => FatValue::EndOfChain,
            value => FatValue::Data(value as u32),
        }
    }
}

pub(crate) type FatTable = Vec<FatValue>;
