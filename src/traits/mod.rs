mod core_read;
mod core_write;

#[cfg(feature = "embedded-hal")]
mod impl_embedded_hal;

pub use self::core_read::{CoreRead, CoreReadBytes, SliceReadError};
pub use self::core_write::CoreWrite;
