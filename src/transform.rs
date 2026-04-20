use std::{
    io::{self, Write},
    marker::PhantomData,
};

use crate::{
    ToScad,
    dim::{_2D, _3D, Dimension, Valid},
    impl_shape_2d, impl_shape_3d,
    shape::Shape,
    shape2d::Shape2d,
    shape3d::Shape3d,
};

#[derive(Clone, Debug)]
pub struct Scaled<D, T> {
    inner: T,
    scale: f64,
    _d: PhantomData<D>,
}
impl_shape_2d!(impl[T: Shape2d] for Scaled<_2D, T>);
impl_shape_3d!(impl[T: Shape3d] for Scaled<_3D, T>);

impl<D, T> Scaled<D, T> {
    pub(crate) fn new(inner: T, scale: f64) -> Self {
        Self {
            inner,
            scale,
            _d: PhantomData,
        }
    }
}

impl<D: Dimension, T> ToScad for Scaled<D, T>
where
    T: ToScad,
    Shape<D, T>: Valid,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "scale({})", self.scale)?;
        self.inner.to_scad(writer)
    }
}

#[derive(Clone, Debug)]
pub struct Translated<D: Dimension, T> {
    inner: T,
    translation: D::Vector,
}
impl_shape_2d!(impl[T: Shape2d] for Translated<_2D, T>);
impl_shape_3d!(impl[T: Shape3d] for Translated<_3D, T>);

impl<D: Dimension, T> Translated<D, T> {
    pub(crate) fn new(inner: T, translation: D::Vector) -> Self {
        Self { inner, translation }
    }
}

impl<D: Dimension, T> ToScad for Translated<D, T>
where
    T: ToScad,
    Shape<D, T>: Valid,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "translate(")?;
        self.translation.to_scad(writer)?;
        write!(writer, ")")?;
        self.inner.to_scad(writer)
    }
}

#[derive(Clone, Debug)]
pub struct Rotated<D: Dimension, T> {
    inner: T,
    rotation: D::Rotation,
}
impl_shape_2d!(impl[T: Shape2d] for Rotated<_2D, T>);
impl_shape_3d!(impl[T: Shape3d] for Rotated<_3D, T>);

impl<D: Dimension, T> Rotated<D, T> {
    pub(crate) fn new(inner: T, rotation: D::Rotation) -> Self {
        Self { inner, rotation }
    }
}

impl<D: Dimension, T> ToScad for Rotated<D, T>
where
    T: ToScad,
    Shape<D, T>: Valid,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "rotate(")?;
        self.rotation.to_scad(writer)?;
        write!(writer, ")")?;
        self.inner.to_scad(writer)
    }
}
