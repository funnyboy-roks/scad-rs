use std::io::{self, Write};

use bauer::Builder;

use crate::{
    ToScad,
    boolean::{Difference, Intersection, Union},
    dim::_3D,
    math::{ScadValue, Vector3},
    modifiers::{Disabled, Highlight, ShowOnly, Transparent},
    shape::{ClosureShape, Shape},
    shape2d::Shape2d,
    transform::{Rotated, Scaled, Translated},
};

pub trait Shape3d: ToScad + Sized {
    /// Wrap a [`Shape3d`] so that it can easily be returned from functions.
    ///
    /// ```
    /// # use scad::{dim::_3D, shape3d::{Shape3d, Cylinder, Cube}, shape::Shape};
    /// fn my_custom_shape() -> Shape<_3D, impl Shape3d> {
    ///     Cylinder::with_radius(2, 10)
    ///         .rotate([0, 90, 0])
    ///         .wrap()
    /// }
    /// ```
    fn wrap(self) -> Shape<_3D, Self> {
        Shape::new(self)
    }

    fn disable(self) -> Disabled<_3D, Self> {
        Disabled::new(self)
    }

    fn show_only(self) -> ShowOnly<_3D, Self> {
        ShowOnly::new(self)
    }

    fn highlight(self) -> Highlight<_3D, Self> {
        Highlight::new(self)
    }

    fn transparent(self) -> Transparent<_3D, Self> {
        Transparent::new(self)
    }

    fn translate(self, translation: impl Into<Vector3>) -> Translated<_3D, Self> {
        Translated::new(self, translation.into())
    }

    fn rotate(self, rotation: impl Into<Vector3>) -> Rotated<_3D, Self> {
        Rotated::new(self, rotation.into())
    }

    fn scale(self, scale: impl Into<f64>) -> Scaled<_3D, Self> {
        Scaled::new(self, scale.into())
    }

    fn difference<R>(self, other: R) -> Difference<_3D, Self, R> {
        Difference::new(self, other)
    }

    fn union<R>(self, other: R) -> Union<_3D, Self, R> {
        Union::new(self, other)
    }

    fn intersection<R>(self, other: R) -> Intersection<_3D, Self, R> {
        Intersection::new(self, other)
    }

    fn number_of_segments(self, n: u32) -> impl Shape3d {
        ClosureShape::<_3D, _>::new(move |writer: &mut dyn Write| {
            write!(writer, "let ($fn = {}) {{", n)?;
            self.to_scad(writer)?;
            write!(writer, "}}")
        })
    }
}

impl<T> Shape3d for &T where T: Shape3d {}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_shape_3d_inner {
    // impl['a, T] for Foo<'a, T>
    (impl$([$($tt: tt)*])? for $ty: ty) => {
        impl<$($($tt)*,)? Rhs: $crate::shape3d::Shape3d> std::ops::Sub<Rhs> for $ty {
            type Output = $crate::boolean::Difference<$crate::dim::_3D, Self, Rhs>;

            fn sub(self, rhs: Rhs) -> Self::Output {
                $crate::shape3d::Shape3d::difference(self, rhs)
            }
        }

        impl<$($($tt)*,)? Rhs: $crate::shape3d::Shape3d> std::ops::Add<Rhs> for $ty {
            type Output = $crate::boolean::Union<$crate::dim::_3D, Self, Rhs>;

            fn add(self, rhs: Rhs) -> Self::Output {
                $crate::shape3d::Shape3d::union(self, rhs)
            }
        }

        impl<$($($tt)*,)? Rhs: $crate::shape3d::Shape3d> std::ops::BitOr<Rhs> for $ty {
            type Output = $crate::boolean::Union<$crate::dim::_3D, Self, Rhs>;

            fn bitor(self, rhs: Rhs) -> Self::Output {
                $crate::shape3d::Shape3d::union(self, rhs)
            }
        }

        impl<$($($tt)*,)? Rhs: $crate::shape3d::Shape3d> std::ops::BitAnd<Rhs> for $ty {
            type Output = $crate::boolean::Intersection<$crate::dim::_3D, Self, Rhs>;

            fn bitand(self, rhs: Rhs) -> Self::Output {
                $crate::shape3d::Shape3d::intersection(self, rhs)
            }
        }
    };
}

/// Implement [`Shape3d`] and some binary operations on a struct
#[macro_export]
macro_rules! impl_shape_3d {
    // impl['a, T] for Foo<'a, T>
    (impl$([$($tt: tt)*])? for $ty: ty) => {
        $crate::impl_shape_3d_inner!(impl$([$($tt)*])? for $ty);
        impl$(<$($tt)*>)? $crate::shape3d::Shape3d for $ty {}
        $crate::impl_shape_3d_inner!(impl$([$($tt)*])? for &$ty);
    };
}

#[derive(Debug)]
pub struct Cube {
    size: Vector3,
    center: bool,
}

impl_shape_3d!(impl for Cube);

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

#[derive(Debug, Clone, Copy)]
enum Radius {
    Split { top: ScadValue, bottom: ScadValue },
    Combined(ScadValue),
}

#[derive(Debug, Clone, Copy)]
pub struct Cylinder {
    radius: Radius,
    height: ScadValue,
    center: bool,
}

impl_shape_3d!(impl for Cylinder);

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

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    radius: ScadValue,
}

impl_shape_3d!(impl for Sphere);

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

#[derive(Debug, Clone, Copy)]
pub struct LinearExtrude<T> {
    inner: T,
    height: ScadValue,
}
impl_shape_3d!(impl[T: Shape2d] for LinearExtrude<T>);

impl<T> LinearExtrude<T> {
    pub(crate) fn new(inner: T, height: ScadValue) -> Self {
        Self { inner, height }
    }
}

impl<T> ToScad for LinearExtrude<T>
where
    T: Shape2d,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "linear_extrude(")?;
        self.height.to_scad(writer)?;
        write!(writer, "){{")?;
        self.inner.to_scad(writer)?;
        write!(writer, "}}")
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

impl_shape_3d!(impl for Polyhedron);

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
