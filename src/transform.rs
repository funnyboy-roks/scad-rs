use std::io::{self, Write};

use crate::{
    ToScad, impl_shape_2d, impl_shape_3d,
    math::{ScadValue, Vector2, Vector3},
    shape3d::Shape3d,
};

#[derive(Clone, Debug)]
pub struct Scaled<T> {
    inner: T,
    scale: f64,
}
impl_shape_3d!(Scaled<T>);

impl<T> Scaled<T> {
    pub(crate) fn new(inner: T, scale: f64) -> Self {
        Self { inner, scale }
    }
}

impl<T> ToScad for Scaled<T>
where
    T: ToScad,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "scale({})", self.scale)?;
        self.inner.to_scad(writer)
    }
}

#[derive(Clone, Debug)]
pub struct Translated<T> {
    inner: T,
    translation: Vector3,
}
impl_shape_3d!(Translated<T>);

impl<T> Translated<T> {
    pub(crate) fn new(inner: T, translation: Vector3) -> Self {
        Self { inner, translation }
    }
}

impl<T> ToScad for Translated<T>
where
    T: ToScad,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "translate(")?;
        self.translation.to_scad(writer)?;
        write!(writer, ")")?;
        self.inner.to_scad(writer)
    }
}

#[derive(Clone, Debug)]
pub struct Rotated<T> {
    inner: T,
    rotation: Vector3,
}
impl_shape_3d!(Rotated<T>);

impl<T> Rotated<T> {
    pub(crate) fn new(inner: T, rotation: Vector3) -> Self {
        Self { inner, rotation }
    }
}

impl<T> ToScad for Rotated<T>
where
    T: ToScad,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "rotate(")?;
        self.rotation.to_scad(writer)?;
        write!(writer, ")")?;
        self.inner.to_scad(writer)
    }
}

#[derive(Clone, Debug)]
pub struct Translated2d<T> {
    inner: T,
    translation: Vector2,
}
impl_shape_2d!(Translated2d<T>);

impl<T> Translated2d<T> {
    pub(crate) fn new(inner: T, translation: Vector2) -> Self {
        Self { inner, translation }
    }
}

impl<T> ToScad for Translated2d<T>
where
    T: ToScad,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "translate(")?;
        self.translation.to_scad(writer)?;
        write!(writer, ")")?;
        self.inner.to_scad(writer)
    }
}

#[derive(Clone, Debug)]
pub struct Rotated2d<T> {
    inner: T,
    rotation: ScadValue,
}
impl_shape_2d!(Rotated2d<T>);

impl<T> Rotated2d<T> {
    pub(crate) fn new(inner: T, rotation: ScadValue) -> Self {
        Self { inner, rotation }
    }
}

impl<T> ToScad for Rotated2d<T>
where
    T: ToScad,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "rotate(")?;
        self.rotation.to_scad(writer)?;
        write!(writer, ")")?;
        self.inner.to_scad(writer)
    }
}
