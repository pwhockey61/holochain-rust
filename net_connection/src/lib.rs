extern crate byteorder;
#[macro_use]
extern crate failure;
extern crate rmp;
extern crate rmp_serde;
#[macro_use]
extern crate serde;
extern crate serde_bytes;
#[macro_use]
extern crate serde_derive;

use failure::Error;

pub type NetResult<T> = Result<T, Error>;

pub mod net_connection;
pub mod net_connection_thread;
pub mod protocol;
