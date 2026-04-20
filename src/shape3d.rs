use std::{
    borrow::Cow,
    io::{self, Write},
    ops::{AddAssign, BitAndAssign, SubAssign},
};

use bauer::Builder;

use crate::{
    ToScad,
    boolean::{Difference, DynDifference, DynIntersection, DynUnion, Intersection, Union},
    math::{ScadValue, Vector3},
    modifiers::{Disabled3d, Highlight3d, ShowOnly3d, Transparent3d},
    shape2d::Shape2d,
    transform::{Rotated, Scaled, Translated},
};

pub trait Shape3d: ToScad + Sized {
    fn disable(self) -> Disabled3d<Self> {
        Disabled3d::new(self)
    }

    fn show_only(self) -> ShowOnly3d<Self> {
        ShowOnly3d::new(self)
    }

    fn highlight(self) -> Highlight3d<Self> {
        Highlight3d::new(self)
    }

    fn transparent(self) -> Transparent3d<Self> {
        Transparent3d::new(self)
    }

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

    fn number_of_segments(self, n: u32) -> impl Shape3d {
        ClosureShape3d::new(move |writer: &mut dyn Write| {
            write!(writer, "let ($fn = {}) {{", n)?;
            self.to_scad(writer)?;
            write!(writer, "}}")
        })
    }
}

/// Implement [`Shape3d`] and some binary operations on a struct
#[macro_export]
macro_rules! impl_shape_3d {
    (($($struct: tt)+)$(<$($lt: lifetime),*$(,)? $($gen: ident: $trait: path),*>)?) => {
        impl<$($($lt,)* $($gen,)*)? Rhs: $crate::shape3d::Shape3d> std::ops::Sub<Rhs> for $($struct)+<$($($lt,)* $($gen),*)?>
            $(where $($gen: $trait),*)?
        {
            type Output = $crate::boolean::Difference<Self, Rhs>;

            fn sub(self, rhs: Rhs) -> Self::Output {
                self.difference(rhs)
            }
        }

        impl<$($($lt,)* $($gen,)*)? Rhs: $crate::shape3d::Shape3d> std::ops::Add<Rhs> for $($struct)+<$($($lt,)* $($gen),*)?>
            $(where $($gen: $trait),*)?
        {
            type Output = $crate::boolean::Union<Self, Rhs>;

            fn add(self, rhs: Rhs) -> Self::Output {
                self.union(rhs)
            }
        }

        impl<$($($lt,)* $($gen,)*)? Rhs: $crate::shape3d::Shape3d> std::ops::BitOr<Rhs> for $($struct)+<$($($lt,)* $($gen),*)?>
            $(where $($gen: $trait),*)?
        {
            type Output = $crate::boolean::Union<Self, Rhs>;

            fn bitor(self, rhs: Rhs) -> Self::Output {
                self.union(rhs)
            }
        }

        impl<$($($lt,)* $($gen,)*)? Rhs: $crate::shape3d::Shape3d> std::ops::BitAnd<Rhs> for $($struct)+<$($($lt,)* $($gen),*)?>
            $(where $($gen: $trait),*)?
        {
            type Output = $crate::boolean::Intersection<Self, Rhs>;

            fn bitand(self, rhs: Rhs) -> Self::Output {
                self.intersection(rhs)
            }
        }


        impl<$($($lt,)* $($gen,)*)?> $crate::shape3d::Shape3d for $($struct)+<$($($lt,)* $($gen),*)?>
            $(where $($gen: $trait),*)? {}
    };
    ($struct: ident$(<$($lt: lifetime),*$(,)? $($gen: ident: $trait: path),*>)?) => {
        impl_shape_3d!(($struct)$(<$($lt,)*$($gen: $trait),*>)?);
        impl_shape_3d!((&$struct)$(<$($lt,)*$($gen: $trait),*>)?);
    };
    ($struct: ident$(<$($lt: lifetime),*$(,)? $($gen: ident),*>)?) => {
        impl_shape_3d!($struct$(<$($lt,)*$($gen: $crate::shape3d::Shape3d),*>)?);
    };
}

#[derive(Clone, Debug)]
pub struct RawShape3d<'a>(Cow<'a, str>);
impl_shape_3d!((RawShape3d)<'a, >);
impl_shape_3d!((&RawShape3d)<'a, >);

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
enum Radius {
    Split { top: ScadValue, bottom: ScadValue },
    Combined(ScadValue),
}

#[derive(Debug)]
pub struct Cylinder {
    radius: Radius,
    height: ScadValue,
    center: bool,
}

impl_shape_3d!(Cylinder);

impl Cylinder {
    pub fn with_radius(radius: impl Into<ScadValue>, height: impl Into<ScadValue>) -> Self {
        Self {
            radius: Radius::Combined(radius.into()),
            height: height.into(),
            center: false,
        }
    }

    pub fn with_diameter(diameter: impl Into<ScadValue>, height: impl Into<ScadValue>) -> Self {
        Self::with_radius(diameter.into() / 2., height)
    }

    pub fn with_radii(
        bottom_r: impl Into<ScadValue>,
        top_r: impl Into<ScadValue>,
        height: impl Into<ScadValue>,
    ) -> Self {
        Self {
            radius: Radius::Split {
                top: top_r.into(),
                bottom: bottom_r.into(),
            },
            height: height.into(),
            center: false,
        }
    }

    pub fn with_diameters(
        bottom_d: impl Into<ScadValue>,
        top_d: impl Into<ScadValue>,
        height: impl Into<ScadValue>,
    ) -> Self {
        Self::with_radii(bottom_d.into() / 2., top_d.into() / 2., height)
    }

    pub fn center(self) -> Self {
        Self {
            center: true,
            ..self
        }
    }
}

impl ToScad for Cylinder {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "cylinder(h = ")?;
        self.height.to_scad(writer)?;
        match &self.radius {
            Radius::Split { top, bottom } => {
                write!(writer, ", r1 = ")?;
                bottom.to_scad(writer)?;
                write!(writer, ", r2 = ")?;
                top.to_scad(writer)?;
            }
            Radius::Combined(scad_value) => {
                write!(writer, ", r = ")?;
                scad_value.to_scad(writer)?;
            }
        }
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

pub struct ClosureShape3d<C> {
    closure: C,
}
impl_shape_3d!(ClosureShape3d<C: Fn(&mut dyn Write) -> io::Result<()>>);

impl<C> ClosureShape3d<C>
where
    C: Fn(&mut dyn Write) -> io::Result<()>,
{
    pub const fn new(closure: C) -> Self {
        Self { closure }
    }
}

impl<C> ToScad for ClosureShape3d<C>
where
    C: Fn(&mut dyn Write) -> io::Result<()>,
{
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

impl<A> FromIterator<A> for Hull
where
    A: Shape3d + 'static,
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

/// 3d shape for accumulating boolean operations
///
/// ```rust
/// # use scad::{shape3d::{Cube, Sphere, DynShape3d}, ToScad};
/// let mut shape = DynShape3d::new();
///
/// shape += Cube::with_size([10, 10, 10]);
/// shape -= Sphere::with_radius(8);
/// shape &= Cube::with_size([12, 5, 12]);
/// # shape.to_scad(&mut std::io::empty()).unwrap();
/// ```
#[derive(Default)]
pub struct DynShape3d {
    inner: Option<Box<dyn ToScad>>,
}
impl_shape_3d!(DynShape3d);

impl ToScad for DynShape3d {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        match &self.inner {
            Some(inner) => inner.to_scad(writer),
            None => panic!("to_scad called on empty DynShape3d"),
        }
    }
}

impl DynShape3d {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T> AddAssign<T> for DynShape3d
where
    T: Shape3d + 'static,
{
    fn add_assign(&mut self, rhs: T) {
        self.inner = match self.inner.take() {
            None => Some(Box::new(rhs)),
            Some(inner) => {
                // SAFETY: rhs is required to be Shape3d by impl bound and self.inner is always constructed
                // using these methods
                let new = unsafe { DynUnion::pair_raw(inner, Box::new(rhs)) };
                Some(Box::new(new))
            }
        }
    }
}

impl<T> SubAssign<T> for DynShape3d
where
    T: Shape3d + 'static,
{
    fn sub_assign(&mut self, rhs: T) {
        self.inner = match self.inner.take() {
            None => None,
            Some(inner) => {
                // SAFETY: rhs is required to be Shape3d by impl bound and self.inner is always constructed
                // using these methods
                let new = unsafe { DynDifference::pair_raw(inner, Box::new(rhs)) };
                Some(Box::new(new))
            }
        }
    }
}

impl<T> BitAndAssign<T> for DynShape3d
where
    T: Shape3d + 'static,
{
    fn bitand_assign(&mut self, rhs: T) {
        self.inner = match self.inner.take() {
            None => None,
            Some(inner) => {
                // SAFETY: rhs is required to be Shape3d by impl bound and self.inner is always constructed
                // using these methods
                let new = unsafe { DynIntersection::pair_raw(inner, Box::new(rhs)) };
                Some(Box::new(new))
            }
        }
    }
}

/// https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Primitive_Solids#polyhedron
#[derive(Debug, Builder)]
pub struct Polyhedron {
    #[builder(into, repeat, repeat_n = 1..)]
    points: Vec<Vector3>,
    #[builder(into, repeat)]
    faces: Vec<Vec<usize>>,
    convexity: Option<u32>,
}

impl_shape_3d!(Polyhedron);

impl ToScad for Polyhedron {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "polyhedron(points=")?;
        self.points.to_scad(writer)?;
        if !self.faces.is_empty() {
            write!(writer, ", faces=")?;
            self.faces.to_scad(writer)?;
        }
        if let Some(conv) = self.convexity {
            write!(writer, ", convexity=")?;
            conv.to_scad(writer)?;
        }
        write!(writer, ");")
    }
}
