#![warn(missing_docs)]
#![no_std]

//! Embedded bincode
//!
//! This crate allows [bincode] to be used on embedded devices that run in `#![no_std]`.
//!
//! Currently this is not possible because bincode requires that the given types implement
//! `std::io::Write` or `std::io::Read`, and bincode supports (de)serializing `alloc` types
//! like `Vec` and `String`.
//!
//! This crate is an alternative (but mostly similar) for bincode that works on microcontrollers.
//! It does this by not supporting types like `Vec` and `String`.
//!
//! Types like `&str` and `&[u8]` are supported. This is possible because `CoreRead` has a
//! requirement that the data being read, has to be persisted somewhere. Usually this is done by a
//! fixed-size backing array. The `&str` and `&[u8]` then simply point to a position in that
//! buffer.

pub mod buffer_writer;
pub mod config;
pub mod deserialize;
pub mod serialize;
pub mod traits;

use self::traits::{CoreRead, CoreWrite};
