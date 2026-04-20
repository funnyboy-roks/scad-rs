use std::{
    fs::File,
    io::{self, BufWriter},
};

use scad::{
    Scad, ToScad,
    shape3d::{Cylinder, Shape3d},
};

fn main() -> io::Result<()> {
    let out = File::create("out.scad")?;
    let mut out = BufWriter::new(out);

    let cup = Cylinder::with_radius(20, 20) - Cylinder::with_radius(19.5, 20).translate((0, 0, 1));

    Scad::builder()
        .number_of_segments(200)
        .objects(&cup)
        .build()
        .to_scad(&mut out)?;

    Ok(())
}
