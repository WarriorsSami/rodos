use std::error::Error;

pub(crate) mod cat;
pub(crate) mod cp;
pub(crate) mod create;
pub(crate) mod del;
pub(crate) mod help;
pub(crate) mod ls;
pub(crate) mod neofetch;
pub(crate) mod rename;

pub(crate) type Void = Result<(), Box<dyn Error>>;
