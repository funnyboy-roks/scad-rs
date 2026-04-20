use std::{
    io::{self, Write},
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::ToScad;

macro_rules! impl_bin_op {
    ($op_name: ident, $func: ident, $op: tt, $name: ident <$($fields: ident),+>) => {
        impl<T> $op_name<T> for $name
        where
            T: Into<ScadValue>,
        {
            type Output = Self;

            fn $func(self, rhs: T) -> Self::Output {
                let rhs = rhs.into();
                Self::new($(self.$fields $op rhs),+)
            }
        }
    };
}

macro_rules! impl_vec {
    (pub struct $name: ident < $($fname: ident = $gname: ident),+ >) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name {
            $(pub $fname: ScadValue),*
        }

        impl $name {
            const ELEMENTS: usize = const { [$(stringify!($fname)),*].len() };

            pub fn new($($fname: impl Into<ScadValue>),*) -> Self {
                Self {
                    $($fname: $fname.into()),*
                }
            }

            pub const fn new_const($($fname: ScadValue),*) -> Self {
                Self { $($fname),* }
            }
        }

        impl<T> From<T> for $name
        where
            T: Into<ScadValue>
        {
            fn from(value: T) -> Self {
                let value = value.into();
                Self {
                    $($fname: value),+
                }
            }
        }

        impl<$($gname),+> From<($($gname),+)> for $name
        where
            $($gname: Into<ScadValue>),+
        {
            fn from(($($fname),+): ($($gname),+)) -> Self {
                Self::new($($fname),+)
            }
        }

        impl<T> From<[T; Self::ELEMENTS]> for $name
        where
            T: Into<ScadValue>
        {
            fn from([$($fname),+]: [T; Self::ELEMENTS]) -> Self {
                Self::new($($fname),+)
            }
        }

        impl ToScad for $name {
            fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
                write!(writer, "[")?;
                $(
                self.$fname.to_scad(writer)?;
                write!(writer, ", ")?;
                )+
                write!(writer, "]")
            }
        }

        impl Add<$name> for $name {
            type Output = Self;

            fn add(self, rhs: $name) -> Self::Output {
                Self::new($(self.$fname + rhs.$fname),*)
            }
        }

        impl Sub<$name> for $name {
            type Output = Self;

            fn sub(self, rhs: $name) -> Self::Output {
                Self::new($(self.$fname - rhs.$fname),*)
            }
        }

        impl_bin_op!(Div, div, /, $name <$($fname),+>);
        impl_bin_op!(Mul, mul, *, $name <$($fname),+>);
        impl_bin_op!(Sub, sub, -, $name <$($fname),+>);
        impl_bin_op!(Add, add, +, $name <$($fname),+>);
    };
}

impl_vec!(pub struct Vector2 <x = X, y = Y>);
impl_vec!(pub struct Vector3 <x = X, y = Y, z = Z>);

/// Define variables for the generated scad file.  These can be modified in the OpenSCAD UI.
///
/// If a doc comment is added to a variable, that documentation will show in the OpenSCAD UI.
///
/// If the final `=> [ident]` is provided, an array is created that holds all variables so they can
/// be added to the Scad builder.
///
/// ```
/// # use scad::{var, Scad, shape3d::Cube};
/// var! {
///     /// Some description
///     let len = 300;
///     let height = 42;
///     => vars
/// }
///
/// let cube = Cube::with_size((len, 10, height));
///
/// Scad::builder()
///     .objects(&cube)
///     .variables(vars)
///     .build();
/// ```
#[macro_export]
macro_rules! var {
    (
        $(
            $(#[doc = $doc: literal])?
            let $($)?$name: ident = $value: expr;
        )+
        $(=> $dest: ident)?
    ) => {
        $(
            let $name = $crate::math::Variable::new(stringify!($name), $value as f64, concat!("", $($doc)?));
        )*
        let _ = $(0; let $dest = )? [$($name),*];
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
        arg0: Option<&'static ScadValue>,
        arg1: &'static ScadValue,
        op: &'static str,
    },
}

impl ScadValue {
    fn op(self, op: &'static str, rhs: ScadValue) -> Self {
        Self::Expression {
            arg0: Some(self.into_static()),
            arg1: rhs.into_static(),
            op,
        }
    }

    fn prefix_op(self, op: &'static str) -> Self {
        Self::Expression {
            arg0: None,
            arg1: self.into_static(),
            op,
        }
    }

    fn into_static(self) -> &'static Self {
        Box::leak(Box::new(self))
    }
}

macro_rules! impl_bin_op {
    ($name: ident, $func: ident, $op: tt, $($nums: ty)+) => {
        impl $name<ScadValue> for ScadValue {
            type Output = ScadValue;

            fn $func(self, rhs: ScadValue) -> Self::Output {
                match (self, rhs) {
                    (ScadValue::Float(l), ScadValue::Float(r)) => ScadValue::Float(l $op r),
                    (l, r) => l.op(stringify!($op), r),
                }
            }
        }

        $(
        impl $name<$nums> for ScadValue {
            type Output = ScadValue;

            fn $func(self, rhs: $nums) -> Self::Output {
                match self {
                    ScadValue::Float(l) => ScadValue::Float(l $op (rhs as f64)),
                    l => l.op(stringify!($op), rhs.into()),
                }
            }
        }

        impl $name<ScadValue> for $nums {
            type Output = ScadValue;

            fn $func(self, rhs: ScadValue) -> Self::Output {
                match rhs {
                    ScadValue::Float(r) => ScadValue::Float((self as f64) $op r),
                    r => ScadValue::from(self).op(stringify!($op), r.into()),
                }
            }
        }
        )+

        impl<T> $name<T> for Variable where T: Into<ScadValue> {
            type Output = ScadValue;

            fn $func(self, rhs: T) -> Self::Output {
                ScadValue::Variable(self).op(stringify!($op), rhs.into())
            }
        }
    };
}

macro_rules! impl_unary_op {
    ($name: ident, $func: ident, $op: tt) => {
        impl $name for ScadValue {
            type Output = ScadValue;

            fn $func(self) -> Self::Output {
                match self {
                    ScadValue::Float(l) => ScadValue::Float($op l),
                    l => l.prefix_op(stringify!($op)),
                }
            }
        }

        impl $name for Variable {
            type Output = ScadValue;

            fn $func(self) -> Self::Output {
                ScadValue::Variable(self).prefix_op(stringify!($op))
            }
        }
    };
}

impl_bin_op!(Div, div, /, u8 u16 u32 u64 usize i8 i16 i32 i64 isize f32 f64);
impl_bin_op!(Mul, mul, *, u8 u16 u32 u64 usize i8 i16 i32 i64 isize f32 f64);
impl_bin_op!(Sub, sub, -, u8 u16 u32 u64 usize i8 i16 i32 i64 isize f32 f64);
impl_bin_op!(Add, add, +, u8 u16 u32 u64 usize i8 i16 i32 i64 isize f32 f64);
impl_unary_op!(Neg, neg, -);

macro_rules! impl_from {
    ($($nums: ty)+) => {
        $(
        impl From<$nums> for ScadValue {
            fn from(value: $nums) -> Self {
                Self::Float(value as f64)
            }
        }
        )+
    };
}

impl_from!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize f32 f64);

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
                write!(writer, "(")?;
                if let Some(arg0) = arg0 {
                    write!(writer, "(")?;
                    arg0.to_scad(writer)?;
                    write!(writer, ")")?;
                }
                write!(writer, "{}(", op)?;
                arg1.to_scad(writer)?;
                write!(writer, "))")
            }
        }
    }
}
