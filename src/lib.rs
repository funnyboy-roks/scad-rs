use std::{
    io::{self, Write},
    path::PathBuf,
};

pub mod boolean;
pub mod math;
pub mod modifiers;
pub mod shape2d;
pub mod shape3d;
pub mod transform;

pub trait ToScad {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()>;
}

impl<T> ToScad for &T
where
    T: ToScad,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        (*self).to_scad(writer)
    }
}

macro_rules! impl_toscad {
    ($ty: ty as Display) => {
        impl ToScad for $ty {
            fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
                write!(writer, "{}", self)
            }
        }
    };
    ($ty: ty as Debug) => {
        impl ToScad for $ty {
            fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
                write!(writer, "{:?}", self)
            }
        }
    };
}

impl_toscad!(str as Debug);
impl_toscad!(u8 as Display);
impl_toscad!(u16 as Display);
impl_toscad!(u32 as Display);
impl_toscad!(u64 as Display);
impl_toscad!(usize as Display);
impl_toscad!(i8 as Display);
impl_toscad!(i16 as Display);
impl_toscad!(i32 as Display);
impl_toscad!(i64 as Display);
impl_toscad!(isize as Display);

impl<T> ToScad for Vec<T>
where
    T: ToScad,
{
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "[")?;
        for (i, v) in self.iter().enumerate() {
            if i > 0 {
                write!(writer, ",")?;
            }
            v.to_scad(writer)?;
        }
        write!(writer, "]")
    }
}

pub struct Raw<'a>(&'a str);

impl<'a> Raw<'a> {
    pub fn new(raw: &'a str) -> Self {
        Self(raw)
    }
}

impl<'a> From<&'a str> for Raw<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl<'a> ToScad for Raw<'a> {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(self.0.as_bytes())
    }
}

#[derive(bauer::Builder)]
pub struct Scad {
    /// Import modules and functions using the `use` keyword
    ///
    /// See <https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Include_Statement>
    #[builder(repeat, into)]
    uses: Vec<PathBuf>,
    /// `$fn`
    ///
    /// The number of segments to use when rendering a circle
    #[builder(default = "0")]
    number_of_segments: u32,
    #[builder(repeat)]
    parameters: Vec<(String, f64)>,
    #[builder(repeat)]
    objects: Vec<Box<dyn ToScad>>,
}

impl ToScad for Scad {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        for path in &self.uses {
            if let Some(path) = path.to_str() {
                writeln!(writer, "use <{}>", path)?;
            } else {
                eprintln!("WARNING: Invalid Path: {}", path.display());
            }
        }

        for (name, default) in &self.parameters {
            writeln!(writer, "{} = {};", name, default)?;
        }

        if self.number_of_segments != 0 {
            writeln!(writer, "$fn = {};", self.number_of_segments)?;
        }

        for o in &self.objects {
            o.to_scad(writer)?;
            writeln!(writer)?;
        }

        Ok(())
    }
}
