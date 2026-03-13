use std::{
    borrow::Cow,
    io::{self, Write},
};

use crate::{
    ToScad,
    boolean::{Difference, Intersection, Union},
    math::{ScadValue, Vector3},
    shape2d::Shape2d,
    transform::{Rotated, Scaled, Translated},
};

pub trait Shape3d: ToScad + Sized {
    fn translate(self, translation: impl Into<Vector3>) -> Translated<Self> {
        Translated::new(self, translation.into())
    }

    fn rotate(self, rotation: impl Into<Vector3>) -> Rotated<Self> {
        Rotated::new(self, rotation.into())
    }

    fn scale(self, scale: impl Into<f64>) -> Scaled<Self> {
        Scaled::new(self, scale.into())
    }

    fn difference<R>(self, other: R) -> Difference<Self, R> {
        Difference::new(self, other)
    }

    fn union<R>(self, other: R) -> Union<Self, R> {
        Union::new(self, other)
    }

    fn intersection<R>(self, other: R) -> Intersection<Self, R> {
        Intersection::new(self, other)
    }
}

/// Implement [`Shape3d`] and some binary operations on a struct
#[macro_export]
macro_rules! impl_shape_3d {
    ($struct: ident$(<$($lt: lifetime),*$(,)? $($gen: ident: $trait: path),*>)?) => {
        impl<$($($lt,)* $($gen,)*)? Rhs: $crate::shape3d::Shape3d> std::ops::Sub<Rhs> for $struct<$($($lt,)* $($gen),*)?>
            $(where $($gen: $trait),*)?
        {
            type Output = $crate::boolean::Difference<Self, Rhs>;

            fn sub(self, rhs: Rhs) -> Self::Output {
                self.difference(rhs)
            }
        }

        impl<$($($lt,)* $($gen,)*)? Rhs: $crate::shape3d::Shape3d> std::ops::Add<Rhs> for $struct<$($($lt,)* $($gen),*)?>
            $(where $($gen: $trait),*)?
        {
            type Output = $crate::boolean::Union<Self, Rhs>;

            fn add(self, rhs: Rhs) -> Self::Output {
                self.union(rhs)
            }
        }

        impl<$($($lt,)* $($gen,)*)? Rhs: $crate::shape3d::Shape3d> std::ops::BitOr<Rhs> for $struct<$($($lt,)* $($gen),*)?>
            $(where $($gen: $trait),*)?
        {
            type Output = $crate::boolean::Union<Self, Rhs>;

            fn bitor(self, rhs: Rhs) -> Self::Output {
                self.union(rhs)
            }
        }

        impl<$($($lt,)* $($gen,)*)? Rhs: $crate::shape3d::Shape3d> std::ops::BitAnd<Rhs> for $struct<$($($lt,)* $($gen),*)?>
            $(where $($gen: $trait),*)?
        {
            type Output = $crate::boolean::Intersection<Self, Rhs>;

            fn bitand(self, rhs: Rhs) -> Self::Output {
                self.intersection(rhs)
            }
        }


        impl<$($($lt,)* $($gen,)*)?> $crate::shape3d::Shape3d for $struct<$($($lt,)* $($gen),*)?>
            $(where $($gen: $trait),*)? {}
    };
    ($struct: ident$(<$($lt: lifetime),*$(,)? $($gen: ident),*>)?) => {
        impl_shape_3d!($struct$(<$($lt,)*$($gen: $crate::shape3d::Shape3d),*>)?);
    };
}

pub struct RawShape3d<'a>(Cow<'a, str>);
impl_shape_3d!(RawShape3d<'a>);

impl<'a> RawShape3d<'a> {
    pub fn new(raw: Cow<'a, str>) -> Self {
        Self(raw)
    }
}

impl<'a> From<&'a str> for RawShape3d<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(Cow::Borrowed(value))
    }
}

impl From<String> for RawShape3d<'static> {
    fn from(value: String) -> Self {
        Self::new(Cow::Owned(value))
    }
}

impl<'a> ToScad for RawShape3d<'a> {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(self.0.as_bytes())
    }
}

#[derive(Debug)]
pub struct Cube {
    size: Vector3,
    center: bool,
}

impl_shape_3d!(Cube);

impl Cube {
    pub fn with_size(size: impl Into<Vector3>) -> Self {
        Self {
            size: size.into(),
            center: false,
        }
    }

    pub fn center(self) -> Self {
        Self {
            size: self.size,
            center: true,
        }
    }
}

impl ToScad for Cube {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "cube(")?;
        self.size.to_scad(writer)?;
        if self.center {
            write!(writer, ", center = true")?;
        }
        write!(writer, ");")
    }
}

#[derive(Debug)]
pub struct Sphere {
    radius: ScadValue,
}

impl_shape_3d!(Sphere);

impl Sphere {
    pub fn with_radius(radius: impl Into<ScadValue>) -> Self {
        Self {
            radius: radius.into(),
        }
    }

    pub fn with_diameter(diameter: impl Into<ScadValue>) -> Self {
        Self::with_radius(diameter.into() / 2.)
    }
}

impl ToScad for Sphere {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "sphere(")?;
        self.radius.to_scad(writer)?;
        write!(writer, ");")
    }
}

#[derive(Debug)]
pub struct LinearExtrude<T> {
    inner: T,
    height: ScadValue,
}
impl_shape_3d!(LinearExtrude<T: Shape2d>);

impl<T> LinearExtrude<T> {
    pub(crate) fn new(inner: T, height: ScadValue) -> Self {
        Self { inner, height }
    }
}

impl<T> ToScad for LinearExtrude<T>
where
    T: ToScad,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "linear_extrude(")?;
        self.height.to_scad(writer)?;
        write!(writer, "){{")?;
        self.inner.to_scad(writer)?;
        write!(writer, "}}")
    }
}

type ScadClosure = dyn Fn(&mut dyn Write) -> io::Result<()>;
pub struct ClosureShape3d {
    closure: Box<ScadClosure>,
}
impl_shape_3d!(ClosureShape3d);

impl ClosureShape3d {
    pub fn new<C>(closure: C) -> Self
    where
        C: Fn(&mut dyn Write) -> io::Result<()> + 'static,
    {
        Self {
            closure: Box::new(closure),
        }
    }
}

impl ToScad for ClosureShape3d {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        (self.closure)(writer)
    }
}

#[derive(Default)]
pub struct Hull {
    inner: Vec<Box<dyn ToScad>>,
}

impl_shape_3d!(Hull);

impl Hull {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    pub fn add<S>(&mut self, s: S)
    where
        S: Shape3d + 'static,
    {
        self.inner.push(Box::new(s))
    }
}

#[macro_export]
macro_rules! hull {
    [$($e: expr),+$(,)?] => {{
        let mut h = $crate::shape3d::Hull::with_capacity(hull![@count $($e),*]);
        $(h.add($e);)*
        h
    }};
    [@count] => { 0 };
    [@count $e0: expr$(, $e: expr)*] => {
        1 + hull![@count $($e),*]
    }
}

impl ToScad for Hull {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "hull(){{")?;
        for x in &self.inner {
            x.to_scad(writer)?;
        }
        write!(writer, "}}")
    }
}
