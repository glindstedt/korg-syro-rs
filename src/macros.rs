pub use paste::paste;

macro_rules! bounds_check {
    ($i:ident, $lo:expr, $hi:expr) => {
        paste! {
            pub (crate) const [<$i:upper _ERROR_NAME>]: &'static str = stringify! { $i };
        }
        paste! {
            pub (crate) fn [<check_ $i>]($i: u32) -> Result<(), SyroError> {
                if $i > $hi || $i < $lo {
                    return Err(SyroError::OutOfBounds {
                        val: $i,
                        name: [<$i:upper _ERROR_NAME>],
                        lo: $lo,
                        hi: $hi,
                    });
                }
                Ok(())
            }
        }
    };
}

macro_rules! max_check {
    ($i:ident, $hi:expr) => {
        paste! {
            pub (crate) const [<$i:upper _ERROR_NAME>]: &'static str = stringify! { $i };
        }
        paste! {
            pub (crate) fn [<check_ $i>]($i: u32) -> Result<(), SyroError> {
                if $i > $hi {
                    return Err(SyroError::OutOfBounds {
                        val: $i,
                        name: [<$i:upper _ERROR_NAME>],
                        lo: 0,
                        hi: $hi,
                    });
                }
                Ok(())
            }
        }
    };
}
