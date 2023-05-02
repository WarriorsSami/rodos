pub(crate) mod disk_manager;
pub(crate) mod i_disk_manager_impl;

pub(crate) type ByteArray = Vec<u8>;

/// A storage buffer is a vector of byte arrays.
pub(crate) type StorageBuffer = Vec<ByteArray>;
