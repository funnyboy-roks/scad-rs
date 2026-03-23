use crate::{ToScad, impl_shape_2d, impl_shape_3d, shape3d::Shape3d};

macro_rules! impl_modifier {
    ($name2d: ident, $name3d: ident, $symbol: literal) => {
        pub struct $name2d<T>(T);

        impl<T> ToScad for $name2d<T>
        where
            T: ToScad,
        {
            fn to_scad(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
                write!(writer, $symbol)?;
                self.0.to_scad(writer)
            }
        }

        impl_shape_2d!($name2d<T>);

        impl<T> $name2d<T> {
            pub(crate) fn new(inner: T) -> Self {
                Self(inner)
            }
        }

        pub struct $name3d<T>(T);

        impl<T> ToScad for $name3d<T>
        where
            T: ToScad,
        {
            fn to_scad(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
                write!(writer, $symbol)?;
                self.0.to_scad(writer)
            }
        }

        impl_shape_3d!($name3d<T>);

        impl<T> $name3d<T> {
            pub(crate) fn new(inner: T) -> Self {
                Self(inner)
            }
        }
    };
}

impl_modifier!(Disabled2d, Disabled3d, "*");
impl_modifier!(ShowOnly2d, ShowOnly3d, "!");
impl_modifier!(Highlight2d, Highlight3d, "#");
impl_modifier!(Transparent2d, Transparent3d, "#");
