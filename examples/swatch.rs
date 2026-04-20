#![allow(clippy::manual_is_multiple_of)]

use std::{
    fs::File,
    io::{self, BufWriter},
};

use scad::{
    Scad, ToScad,
    boolean::DynUnion,
    impl_shape_3d,
    math::Vector3,
    shape2d::{HorizontalAlign, Shape2d, Text, VerticalAlign},
    shape3d::{Cube, Hull, Shape3d, Sphere},
};

#[derive(Debug)]
struct RoundedCube {
    size: Vector3,
    corner_r: f64,
}

impl RoundedCube {
    fn new(size: impl Into<Vector3>, corner_r: f64) -> Self {
        Self {
            size: size.into(),
            corner_r,
        }
    }
}

impl_shape_3d!(impl for RoundedCube);

// losely based on https://openhome.cc/eGossip/OpenSCAD/lib3x-rounded_cube.html
impl ToScad for RoundedCube {
    fn to_scad(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        // hull of spheres at corners of the cube (inset by radius)

        let edge_len = self.size - Vector3::from(2.) * self.corner_r;

        let h = (0..8)
            .map(|i| {
                let x = (i & 1) as f64;
                let y = ((i & 2) >> 1) as f64;
                let z = ((i & 4) >> 2) as f64;
                let offset = (edge_len.x * x, edge_len.y * y, edge_len.z * z);

                Sphere::with_radius(self.corner_r)
                    .translate(offset)
                    .translate(self.corner_r)
            })
            .collect::<Hull>();

        h.to_scad(writer)
    }
}

fn main() -> io::Result<()> {
    let out = File::create("out.scad")?;
    let mut out = BufWriter::new(out);

    let swatch_size = 24.;
    let thicc = 2.25;
    let text_thicc = 0.5;

    let base = RoundedCube::new((swatch_size, swatch_size, thicc), 1.)
        - Cube::with_size((swatch_size - 4., swatch_size - 4., text_thicc)).translate((
            2,
            2,
            thicc - text_thicc + 0.001,
        ));

    let lines = ["PLA", "Wood"];

    let text_size = 4.;
    let line_height = text_size + 2.;
    let font = "Iosevka:style=Bold";

    let offset_y = (lines.len() / 2) as f64 * line_height
        - if lines.len() % 2 == 0 {
            line_height * 0.5
        } else {
            0.
        };

    let mut union = DynUnion::new();

    for (i, line) in lines.into_iter().enumerate() {
        let text = Text::builder()
            .text(line)
            .size(text_size)
            .font(font)
            .halign(HorizontalAlign::Center)
            .valign(VerticalAlign::Center)
            .build()
            .unwrap()
            .translate([0., offset_y + i as f64 * -line_height]);
        union.add(text);
    }

    let text = union.linear_extrude(text_thicc).translate([
        swatch_size / 2.,
        swatch_size / 2.,
        thicc - text_thicc - 0.1,
    ]);

    Scad::builder()
        .number_of_segments(100)
        .objects(&(base + text))
        .build()
        .to_scad(&mut out)?;

    Ok(())
}
