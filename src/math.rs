use std::{
    io::{self, Write},
    ops::{Add, Div, Mul, Sub},
};

use crate::ToScad;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum ScadValue {
    Float(f64),
    Variable(String),
    Expression {
        args: Box<(ScadValue, ScadValue)>,
        op: &'static str,
    },
}

impl ScadValue {
    pub fn op(self, op: &'static str, rhs: ScadValue) -> Self {
        Self::Expression {
            args: Box::new((self, rhs)),
            op,
        }
    }
}

impl Div<ScadValue> for ScadValue {
    type Output = ScadValue;

    fn div(self, rhs: ScadValue) -> Self::Output {
        match (self, rhs) {
            (ScadValue::Float(l), ScadValue::Float(r)) => ScadValue::Float(l / r),
            (l, r) => l.op("/", r),
        }
    }
}

impl Div<f64> for ScadValue {
    type Output = ScadValue;

    fn div(self, rhs: f64) -> Self::Output {
        match self {
            ScadValue::Float(l) => ScadValue::Float(l / rhs),
            l => l.op("/", rhs.into()),
        }
    }
}

impl Mul<ScadValue> for ScadValue {
    type Output = ScadValue;

    fn mul(self, rhs: ScadValue) -> Self::Output {
        match (self, rhs) {
            (ScadValue::Float(l), ScadValue::Float(r)) => ScadValue::Float(l * r),
            (l, r) => l.op("*", r),
        }
    }
}

impl Mul<f64> for ScadValue {
    type Output = ScadValue;

    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            ScadValue::Float(l) => ScadValue::Float(l * rhs),
            l => l.op("*", rhs.into()),
        }
    }
}

impl Sub<ScadValue> for ScadValue {
    type Output = ScadValue;

    fn sub(self, rhs: ScadValue) -> Self::Output {
        match (self, rhs) {
            (ScadValue::Float(l), ScadValue::Float(r)) => ScadValue::Float(l - r),
            (l, r) => l.op("-", r),
        }
    }
}

impl Sub<f64> for ScadValue {
    type Output = ScadValue;

    fn sub(self, rhs: f64) -> Self::Output {
        match self {
            ScadValue::Float(l) => ScadValue::Float(l - rhs),
            l => l.op("-", rhs.into()),
        }
    }
}

impl Add<ScadValue> for ScadValue {
    type Output = ScadValue;

    fn add(self, rhs: ScadValue) -> Self::Output {
        match (self, rhs) {
            (ScadValue::Float(l), ScadValue::Float(r)) => ScadValue::Float(l + r),
            (l, r) => l.op("+", r),
        }
    }
}

impl Add<f64> for ScadValue {
    type Output = ScadValue;

    fn add(self, rhs: f64) -> Self::Output {
        match self {
            ScadValue::Float(l) => ScadValue::Float(l + rhs),
            l => l.op("+", rhs.into()),
        }
    }
}

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

impl From<&str> for ScadValue {
    fn from(value: &str) -> Self {
        Self::Variable(value.into())
    }
}

impl ToScad for ScadValue {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        match self {
            ScadValue::Float(f) => write!(writer, "{}", f),
            ScadValue::Variable(v) => write!(writer, "{}", v),
            ScadValue::Expression { args, op } => {
                write!(writer, "(")?;
                args.0.to_scad(writer)?;
                write!(writer, "){}(", op)?;
                args.0.to_scad(writer)?;
                write!(writer, ")")
            }
        }
    }
}
