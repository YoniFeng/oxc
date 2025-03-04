//! Indexing Utilities
//!
//! <https://github.com/rust-lang/rust/tree/master/compiler/rustc_index>

mod idx;
mod vec;

pub use idx::{Idx, NonZeroIdx};
pub use vec::IndexVec;

/// Type size assertion. The first argument is a type and the second argument is its expected size.
///
/// <https://github.com/rust-lang/rust/blob/c86e7fb60f5343041fd0c27d4affaf3261115666/compiler/rustc_index/src/lib.rs#L30-L36>
#[macro_export]
macro_rules! static_assert_size {
    ($ty:ty, $size:expr) => {
        const _: [(); $size] = [(); std::mem::size_of::<$ty>()];
    };
}
