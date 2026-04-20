use std::{
    borrow::Cow,
    io::{self, Write},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    ToScad,
    dim::{_2D, _3D, Dimension, Valid},
    impl_shape_2d, impl_shape_3d, sealed,
    shape2d::Shape2d,
    shape3d::Shape3d,
};

pub trait ScadClosure: Fn(&mut dyn Write) -> io::Result<()> + sealed::Sealed {}
impl<T> sealed::Sealed for T where T: Fn(&mut dyn Write) -> io::Result<()> {}
impl<T> ScadClosure for T where T: Fn(&mut dyn Write) -> io::Result<()> {}

pub struct ClosureShape<D, C> {
    closure: C,
    _d: PhantomData<D>,
}

impl_shape_3d!(impl[C: ScadClosure] for ClosureShape<_3D, C>);
impl_shape_2d!(impl[C: ScadClosure] for ClosureShape<_2D, C>);

impl<D, C> ClosureShape<D, C>
where
    C: ScadClosure,
{
    pub fn new(closure: C) -> Self {
        Self {
            closure,
            _d: PhantomData,
        }
    }
}

impl<D, C> ToScad for ClosureShape<D, C>
where
    D: Dimension,
    C: ScadClosure,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        (self.closure)(writer)
    }
}

/// Wrapper type around a `Shape*` that can be used for wrapping `impl Shape*` returned from a
/// function:
///
/// ```
/// # use scad::{dim::_3D, shape3d::{Shape3d, Cylinder, Cube}, shape::Shape};
/// fn my_custom_shape() -> Shape<_3D, impl Shape3d> {
///     Cylinder::with_radius(2, 10)
///         .rotate([0, 90, 0])
///         .wrap()
/// }
/// ```
///
/// The idea is that this will manage the implementation of binary operations so that functions can
/// have inferred return types
pub struct Shape<D, T> {
    inner: T,
    _d: PhantomData<D>,
}

impl<T: Shape2d> sealed::Sealed for Shape<_2D, T> {}
impl<T: Shape2d> Valid for Shape<_2D, T> {}
impl<T: Shape3d> sealed::Sealed for Shape<_3D, T> {}
impl<T: Shape3d> Valid for Shape<_3D, T> {}

impl<D, T> Shape<D, T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _d: PhantomData,
        }
    }
}

impl<D, T> From<T> for Shape<D, T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<D, T> Deref for Shape<D, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D, T> DerefMut for Shape<D, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<D, T> ToScad for Shape<D, T>
where
    T: ToScad,
    Self: Valid,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        self.inner.to_scad(writer)
    }
}

impl_shape_2d!(impl[T: Shape2d] for Shape<_2D, T>);
impl_shape_3d!(impl[T: Shape3d] for Shape<_3D, T>);

pub struct Raw<'a, D> {
    raw: Cow<'a, str>,
    _d: PhantomData<D>,
}

impl_shape_3d!(impl for Raw<'_, _3D>);
impl_shape_2d!(impl for Raw<'_, _2D>);

impl<'a> Raw<'a, _2D> {
    pub fn new_2d(raw: Cow<'a, str>) -> Self {
        Self {
            raw,
            _d: PhantomData,
        }
    }
}

impl<'a> Raw<'a, _3D> {
    pub fn new_3d(raw: Cow<'a, str>) -> Self {
        Self {
            raw,
            _d: PhantomData,
        }
    }
}

impl<'a, D: Dimension> ToScad for Raw<'a, D> {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(self.raw.as_bytes())
    }
}

#[macro_export]
macro_rules! hull {
    [$($e: expr),+$(,)?] => {{
        let mut h = $crate::shape::Hull::with_capacity(
            const {
                [$(stringify!($e)),*].len()
            }
        );
        $(h.add($e);)*
        h
    }};
}

pub struct Hull<D> {
    inner: Vec<Box<dyn ToScad>>,
    _d: PhantomData<D>,
}

impl_shape_3d!(impl for Hull<_3D>);
impl_shape_2d!(impl for Hull<_2D>);

impl<D> Default for Hull<D> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _d: PhantomData,
        }
    }
}

impl<D> Hull<D> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            _d: PhantomData,
        }
    }

    pub fn add<S>(&mut self, s: S)
    where
        S: ToScad + 'static,
        Shape<D, S>: Valid,
    {
        self.inner.push(Box::new(s))
    }
}

impl<D: Dimension> ToScad for Hull<D> {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "hull(){{")?;
        for x in &self.inner {
            x.to_scad(writer)?;
        }
        write!(writer, "}}")
    }
}

impl<D, A> FromIterator<A> for Hull<D>
where
    A: ToScad + 'static,
    Shape<D, A>: Valid,
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let iter = iter.into_iter();

        let mut this = match iter.size_hint() {
            (_, Some(h)) => Self::with_capacity(h),
            (l, None) => Self::with_capacity(l),
        };

        for x in iter {
            this.add(x)
        }

        this
    }
}
