use std::error::Error;

pub(crate) mod create;
pub(crate) mod ls;
pub(crate) mod neofetch;
pub(crate) mod rename;

pub(crate) type Void = Result<(), Box<dyn Error>>;
