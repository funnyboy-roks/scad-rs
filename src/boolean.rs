use std::{
    fmt::Debug,
    io::{self, Write},
    marker::PhantomData,
    ops::{AddAssign, BitAndAssign, SubAssign},
};

use crate::{
    ToScad,
    dim::{_2D, _3D, Dimension, Valid},
    impl_shape_2d, impl_shape_3d,
    shape::Shape,
    shape2d::Shape2d,
    shape3d::Shape3d,
};

/// Implement the struct for a boolean operation
macro_rules! impl_boolean {
    ($name: ident, $fn: literal) => {
        #[derive(Debug)]
        pub struct $name<D, T, U> {
            left: T,
            right: U,
            _d: PhantomData<D>,
        }

        impl<D, T, U> $name<D, T, U> {
            pub(crate) fn new(left: T, right: U) -> Self {
                Self {
                    left,
                    right,
                    _d: PhantomData,
                }
            }
        }

        impl<D, T, U> ToScad for $name<D, T, U>
        where
            T: ToScad,
            U: ToScad,
            Shape<D, T>: Valid,
            Shape<D, U>: Valid,
        {
            fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
                write!(writer, concat!($fn, "(){{"))?;
                self.left.to_scad(writer)?;
                self.right.to_scad(writer)?;
                write!(writer, "}}")
            }
        }

        impl_shape_2d!(impl[T: Shape2d, U: Shape2d] for $name<_2D, T, U>);
        impl_shape_3d!(impl[T: Shape3d, U: Shape3d] for $name<_3D, T, U>);
    };
}

macro_rules! impl_dyn_boolean {
    ($name: ident, $fn: literal, $op: ident, $op_fn: ident) => {
        pub struct $name<D> {
            items: Vec<Box<dyn ToScad>>,
            _d: PhantomData<D>,
        }

        impl<D> Default for $name<D> {
            fn default() -> Self {
                Self {
                    items: Default::default(),
                    _d: PhantomData
                }
            }
        }

        impl Debug for $name<_2D> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name<_2D>)).finish_non_exhaustive()
            }
        }

        impl Debug for $name<_3D> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name<_3D>)).finish_non_exhaustive()
            }
        }

        impl<D> $name<D> {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn with_capacity(capacity: usize) -> Self {
                Self {
                    items: Vec::with_capacity(capacity),
                    _d: PhantomData
                }
            }
        }

        impl<D> $name<D> {
            pub fn pair<L, R>(lhs: L, rhs: R) -> Self
            where
                L: ToScad + 'static,
                R: ToScad + 'static,
                Shape<D, L>: Valid,
                Shape<D, R>: Valid,
            {
                // SAFETY: Type is enforced by the signature
                unsafe { Self::pair_raw(Box::new(lhs), Box::new(rhs)) }
            }

            #[doc = "# SAFETY"]
            #[doc = ""]
            #[doc = concat!("The caller must ensure that lhs and rhs are both [`", stringify!($trait), "`].")]
            pub unsafe fn pair_raw(lhs: Box<dyn ToScad>, rhs: Box<dyn ToScad>) -> Self
            {
                Self {
                    items: vec![lhs, rhs],
                    _d: PhantomData,
                }
            }

            pub fn add<S>(&mut self, s: S)
            where
                S: ToScad + 'static,
                Shape<D, S>: Valid,
            {
                self.items.push(Box::new(s));
            }
        }

        impl<D, A> FromIterator<A> for $name<D>
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

        impl<D, T> $op<T> for $name<D>
        where
            T: ToScad + 'static,
            Shape<D, T>: Valid,
        {
            fn $op_fn(&mut self, rhs: T) {
                self.add(rhs)
            }
        }

        impl<D: Dimension> ToScad for $name<D> {
            fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
                write!(writer, concat!($fn, "(){{"))?;
                for item in &self.items {
                    item.to_scad(writer)?;
                }
                write!(writer, "}}")
            }
        }

        impl_shape_3d!(impl for $name<_3D>);
        impl_shape_2d!(impl for $name<_2D>);
    };
}

impl_boolean!(Difference, "difference");
impl_boolean!(Union, "union");
impl_boolean!(Intersection, "intersection");

impl_dyn_boolean!(DynDifference, "difference", SubAssign, sub_assign);
impl_dyn_boolean!(DynUnion, "union", AddAssign, add_assign);
impl_dyn_boolean!(DynIntersection, "intersection", BitAndAssign, bitand_assign);

#[macro_export]
macro_rules! union {
    [$($expr: expr),* $(,)?] => {{
        let mut u = $crate::boolean::DynUnion::with_capacity(
            const {
                [$(stringify!($expr)),*].len()
            }
        );
        $(u.add($expr);)*
        u
    }};
}

#[macro_export]
macro_rules! intersection {
    [$($expr: expr),* $(,)?] => {{
        let mut i = $crate::boolean::DynIntersection::with_capacity(
            const {
                [$(stringify!($expr)),*].len()
            }
        );
        $(i.add($expr);)*
        i
    }};
}

/// Shape for accumulating boolean operations
///
/// ```
/// # use scad::{shape3d::{Cube, Sphere}, ToScad, shape::DynShape};
/// let mut shape = DynShape::new();
///
/// shape += Cube::with_size([10, 10, 10]);
/// shape -= Sphere::with_radius(8);
/// shape &= Cube::with_size([12, 5, 12]);
/// # shape.to_scad(&mut std::io::empty()).unwrap();
/// ```
pub struct DynShape<D> {
    inner: Option<Box<dyn ToScad>>,
    _d: PhantomData<D>,
}
impl_shape_2d!(impl for DynShape<_2D>);
impl_shape_3d!(impl for DynShape<_3D>);

impl<D: Dimension> ToScad for DynShape<D> {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        match &self.inner {
            Some(inner) => inner.to_scad(writer),
            None => panic!("to_scad called on empty DynShape3d"),
        }
    }
}

impl<D> Default for DynShape<D> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _d: PhantomData,
        }
    }
}

impl<D> DynShape<D> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_none()
    }
}

impl<D, T> AddAssign<T> for DynShape<D>
where
    T: ToScad + 'static,
    Shape<D, T>: Valid,
{
    fn add_assign(&mut self, rhs: T) {
        self.inner = match self.inner.take() {
            None => Some(Box::new(rhs)),
            Some(inner) => {
                // SAFETY: rhs is required to be Shape3d by impl bound and self.inner is always constructed
                // using these methods
                let new = unsafe { DynUnion::<_3D>::pair_raw(inner, Box::new(rhs)) };
                Some(Box::new(new))
            }
        }
    }
}

impl<D, T> SubAssign<T> for DynShape<D>
where
    T: ToScad + 'static,
    Shape<D, T>: Valid,
{
    fn sub_assign(&mut self, rhs: T) {
        self.inner = match self.inner.take() {
            None => panic!("Attempt to subtract from empty DynShape"),
            Some(inner) => {
                // SAFETY: rhs is required to be Shape3d by impl bound and self.inner is always constructed
                // using these methods
                let new = unsafe { DynDifference::<_3D>::pair_raw(inner, Box::new(rhs)) };
                Some(Box::new(new))
            }
        }
    }
}

impl<D, T> BitAndAssign<T> for DynShape<D>
where
    T: ToScad + 'static,
    Shape<D, T>: Valid,
{
    fn bitand_assign(&mut self, rhs: T) {
        self.inner = match self.inner.take() {
            None => None,
            Some(inner) => {
                // SAFETY: rhs is required to be Shape3d by impl bound and self.inner is always constructed
                // using these methods
                let new = unsafe { DynIntersection::<_3D>::pair_raw(inner, Box::new(rhs)) };
                Some(Box::new(new))
            }
        }
    }
}
