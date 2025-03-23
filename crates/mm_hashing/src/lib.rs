pub use paste::paste;

mod hash_list;
pub use hash_list::HashList;

mod hash_string_macros;

mod hash_string;
pub use hash_string::HashString;

mod little32;
pub use little32::hash as hash_little32;

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! const_assert {
        ($($tt:tt)*) => {
            const _: () = assert!($($tt)*);
        }
    }
}
