use std::{
    fmt::Debug,
    io::{self, Write},
};

use crate::{ToScad, impl_shape_2d, impl_shape_3d, shape2d::Shape2d, shape3d::Shape3d};

/// Implement the struct for a boolean operation
macro_rules! impl_boolean {
    ($name: ident, $fn: literal, $trait: ident) => {
        #[derive(Debug)]
        pub struct $name<T, U>(T, U);

        impl<T, U> $name<T, U> {
            pub(crate) fn new(left: T, right: U) -> Self {
                Self(left, right)
            }
        }

        impl<T, U> ToScad for $name<T, U>
        where
            T: $trait,
            U: $trait,
        {
            fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
                write!(writer, concat!($fn, "(){{"))?;
                self.0.to_scad(writer)?;
                self.1.to_scad(writer)?;
                write!(writer, "}}")
            }
        }
    };
}

macro_rules! impl_dyn_boolean {
    ($name: ident, $fn: literal, $trait: ident) => {
        #[derive(Default)]
        pub struct $name {
            items: Vec<Box<dyn ToScad>>,
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }

        impl $name {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn with_capacity(capacity: usize) -> Self {
                Self {
                    items: Vec::with_capacity(capacity),
                }
            }

            pub fn add<S>(&mut self, s: S)
            where
                S: $trait + 'static,
            {
                self.items.push(Box::new(s));
            }
        }

        impl ToScad for $name {
            fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
                write!(writer, concat!($fn, "(){{"))?;
                for item in &self.items {
                    item.to_scad(writer)?;
                }
                write!(writer, "}}")
            }
        }
    };
}

impl_boolean!(Difference, "difference", Shape3d);
impl_shape_3d!(Difference<T, U>);
impl_boolean!(Union, "union", Shape3d);
impl_shape_3d!(Union<T, U>);
impl_boolean!(Intersection, "intersection", Shape3d);
impl_shape_3d!(Intersection<T, U>);

impl_boolean!(Difference2d, "difference", Shape2d);
impl_shape_2d!(Difference2d<T, U>);
impl_boolean!(Union2d, "union", Shape2d);
impl_shape_2d!(Union2d<T, U>);
impl_boolean!(Intersection2d, "intersection", Shape2d);
impl_shape_2d!(Intersection2d<T, U>);

impl_dyn_boolean!(DynDifference, "difference", Shape3d);
impl_shape_3d!(DynDifference);
impl_dyn_boolean!(DynUnion, "union", Shape3d);
impl_shape_3d!(DynUnion);
impl_dyn_boolean!(DynIntersection, "intersection", Shape3d);
impl_shape_3d!(DynIntersection);

impl_dyn_boolean!(DynDifference2d, "difference", Shape2d);
impl_shape_2d!(DynDifference2d);
impl_dyn_boolean!(DynUnion2d, "union", Shape2d);
impl_shape_2d!(DynUnion2d);
impl_dyn_boolean!(DynIntersection2d, "intersection", Shape2d);
impl_shape_2d!(DynIntersection2d);
