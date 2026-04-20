use std::{
    io::{self, Write},
    ops::{Add, Div, Mul, Sub},
};

use crate::ToScad;

#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: ScadValue,
    pub y: ScadValue,
    pub z: ScadValue,
}

impl From<f64> for Vector3 {
    fn from(value: f64) -> Self {
        Self::new(value, value, value)
    }
}

impl<T> From<[T; 3]> for Vector3
where
    T: Into<ScadValue>,
{
    fn from([x, y, z]: [T; 3]) -> Self {
        Self::new(x, y, z)
    }
}

impl<T, U, V> From<(T, U, V)> for Vector3
where
    T: Into<ScadValue>,
    U: Into<ScadValue>,
    V: Into<ScadValue>,
{
    fn from((x, y, z): (T, U, V)) -> Self {
        Self::new(x, y, z)
    }
}

impl Vector3 {
    pub fn new(x: impl Into<ScadValue>, y: impl Into<ScadValue>, z: impl Into<ScadValue>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    pub const fn new_const(x: ScadValue, y: ScadValue, z: ScadValue) -> Self {
        Self { x, y, z }
    }
}

impl Div<f64> for Vector3 {
    type Output = Vector3;

    fn div(self, rhs: f64) -> Self::Output {
        Vector3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Mul<f64> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: f64) -> Self::Output {
        Vector3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Sub for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl ToScad for Vector3 {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "[")?;
        self.x.to_scad(writer)?;
        write!(writer, ", ")?;
        self.y.to_scad(writer)?;
        write!(writer, ", ")?;
        self.z.to_scad(writer)?;
        write!(writer, "]")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: ScadValue,
    pub y: ScadValue,
}

impl From<f64> for Vector2 {
    fn from(value: f64) -> Self {
        Self::new(value, value)
    }
}

impl<T> From<[T; 2]> for Vector2
where
    T: Into<ScadValue>,
{
    fn from([x, y]: [T; 2]) -> Self {
        Self::new(x, y)
    }
}

impl<T, U> From<(T, U)> for Vector2
where
    T: Into<ScadValue>,
    U: Into<ScadValue>,
{
    fn from((x, y): (T, U)) -> Self {
        Self::new(x, y)
    }
}

impl Vector2 {
    pub fn new(x: impl Into<ScadValue>, y: impl Into<ScadValue>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }

    pub const fn new_const(x: ScadValue, y: ScadValue) -> Self {
        Self { x, y }
    }
}

impl ToScad for Vector2 {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "[")?;
        self.x.to_scad(writer)?;
        write!(writer, ", ")?;
        self.y.to_scad(writer)?;
        write!(writer, "]")
    }
}

#[macro_export]
macro_rules! var {
    ($(
        $(#[doc = $doc: literal])?
        let $($)?$name: ident = $value: expr;
    )+) => {
        $(
            let $name = $crate::math::Variable::new(stringify!($name), $value as f64, concat!("", $($doc)?));
        )*
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Variable {
    name: &'static str,
    default_value: f64,
    description: &'static str,
}

impl Variable {
    pub fn new(name: &'static str, default_value: f64, description: &'static str) -> Self {
        Self {
            name,
            default_value,
            description,
        }
    }

    pub fn write_definition(&self, writer: &mut dyn Write) -> io::Result<()> {
        if !self.description.trim().is_empty() {
            writeln!(writer, "// {}", self.description)?;
        }
        writeln!(writer, "{} = {};", self.name, self.default_value)
    }
}

impl ToScad for Variable {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(self.name.as_bytes())
    }
}

/// NOTE: This leaks memory like crazy, but it shouldn't really matter as the intended use-case is
/// in very short batch programs
#[derive(Debug, Clone, Copy)]
pub enum ScadValue {
    Float(f64),
    Variable(Variable),
    Expression {
        arg0: &'static ScadValue,
        arg1: &'static ScadValue,
        op: &'static str,
    },
}

impl ScadValue {
    fn op(self, op: &'static str, rhs: ScadValue) -> Self {
        Self::Expression {
            arg0: self.into_static(),
            arg1: rhs.into_static(),
            op,
        }
    }

    fn into_static(self) -> &'static Self {
        Box::leak(Box::new(self))
    }
}

macro_rules! impl_op {
    ($name: ident, $func: ident, $op: tt) => {
        impl $name<ScadValue> for ScadValue {
            type Output = ScadValue;

            fn $func(self, rhs: ScadValue) -> Self::Output {
                match (self, rhs) {
                    (ScadValue::Float(l), ScadValue::Float(r)) => ScadValue::Float(l $op r),
                    (l, r) => l.op(stringify!($op), r),
                }
            }
        }

        impl $name<f64> for ScadValue {
            type Output = ScadValue;

            fn $func(self, rhs: f64) -> Self::Output {
                match self {
                    ScadValue::Float(l) => ScadValue::Float(l $op rhs),
                    l => l.op(stringify!($op), rhs.into()),
                }
            }
        }

        impl<T> $name<T> for Variable where T: Into<ScadValue> {
            type Output = ScadValue;

            fn $func(self, rhs: T) -> Self::Output {
                ScadValue::Variable(self).op(stringify!($op), rhs.into())
            }
        }
    };
}

impl_op!(Div, div, /);
impl_op!(Mul, mul, *);
impl_op!(Sub, sub, -);
impl_op!(Add, add, +);

impl From<f64> for ScadValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<i32> for ScadValue {
    fn from(value: i32) -> Self {
        Self::Float(value.into())
    }
}

impl From<Variable> for ScadValue {
    fn from(value: Variable) -> Self {
        Self::Variable(value)
    }
}

impl ToScad for ScadValue {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        match self {
            ScadValue::Float(f) => write!(writer, "{}", f),
            ScadValue::Variable(v) => v.to_scad(writer),
            ScadValue::Expression { arg0, arg1, op } => {
                write!(writer, "((")?;
                arg0.to_scad(writer)?;
                write!(writer, "){}(", op)?;
                arg1.to_scad(writer)?;
                write!(writer, "))")
            }
        }
    }
}
