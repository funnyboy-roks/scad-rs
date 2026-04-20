use crate::{
    ToScad,
    dim::{_2D, _3D, Dimension, Valid},
    impl_shape_2d, impl_shape_3d,
    shape::Shape,
    shape2d::Shape2d,
    shape3d::Shape3d,
};
use std::marker::PhantomData;

macro_rules! impl_modifier {
    ($name: ident, $symbol: literal) => {
        pub struct $name<D, T> {
            t: T,
            _d: PhantomData<D>,
        }

        impl<D: Dimension, T> ToScad for $name<D, T>
        where
            T: ToScad,
            Shape<D, T>: Valid,
        {
            fn to_scad(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
                write!(writer, $symbol)?;
                self.t.to_scad(writer)
            }
        }

        impl<D, T> $name<D, T> {
            pub(crate) fn new(inner: T) -> Self {
                Self {
                    t: inner,
                    _d: PhantomData,
                }
            }
        }

        impl_shape_2d!(impl[T: Shape2d] for $name<_2D, T>);
        impl_shape_3d!(impl[T: Shape3d] for $name<_3D, T>);
    };
}

impl_modifier!(Disabled, "*");
impl_modifier!(ShowOnly, "!");
impl_modifier!(Highlight, "#");
impl_modifier!(Transparent, "#");
