
#[macro_export]

macro_rules! bitflags {
    ($vis:vis $name:ident [$($Flag:ident),*]) => {
        
        // #[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
        $vis struct $name(u8);

        impl $name {
            bitflags!(@step 0usize, $($Flag,)*);

            // $(
            //     pub fn $Flag(&self) -> bool {
            //         self.bits & Self::$flag_$Flag != 0
            //     }
            // )*
        }
    };
    (@step $idx:expr, $head:ident, $($tail:ident,)*) => {
        pub fn $head(&self) -> bool {
            ((0x1 << $idx) & self.0) != 0
        }

        bitflags!(@step $idx + 1usize, $($tail,)*);
    };
    (@step $_idx:expr,) => {};
    () => {};
}

