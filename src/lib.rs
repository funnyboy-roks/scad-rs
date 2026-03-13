use std::{
    io::{self, Write},
    path::PathBuf,
};

pub mod boolean;
pub mod math;
pub mod shape2d;
pub mod shape3d;
pub mod transform;

pub trait ToScad {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()>;
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

impl ToScad for str {
    fn to_scad(&self, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "{:?}", self)
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
