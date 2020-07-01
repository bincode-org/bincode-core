mod core_read;
mod core_write;

pub use self::core_read::{CoreRead, SliceReadError};
pub use self::core_write::CoreWrite;
