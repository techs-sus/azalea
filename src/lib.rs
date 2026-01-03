//! Azalea is an experimental rbxm/rbxmx transformation suite.
//!
//! Currently, it is most useful when used to embed models in environments that forbid `require(id)`.

pub mod emit;
pub mod encoder;
pub mod spec;

#[cfg(feature = "base122")]
pub mod base122;
