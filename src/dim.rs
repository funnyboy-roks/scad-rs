use crate::{
    ToScad,
    math::{ScadValue, Vector2, Vector3},
    sealed,
};

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default)]
pub struct _2D;
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default)]
pub struct _3D;

pub trait Dimension: sealed::Sealed {
    type Vector: ToScad;
    type Rotation: ToScad;
}

impl sealed::Sealed for _2D {}
impl Dimension for _2D {
    type Vector = Vector2;
    type Rotation = ScadValue;
}
impl sealed::Sealed for _3D {}
impl Dimension for _3D {
    type Vector = Vector3;
    type Rotation = Vector3;
}

pub trait Valid: sealed::Sealed {}
