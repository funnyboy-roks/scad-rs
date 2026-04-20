use std::{
    fs::File,
    io::{self, BufWriter},
};

use scad::{
    Scad, ToScad,
    boolean::DynUnion,
    hull,
    math::Vector3,
    shape3d::{Cylinder, Shape3d},
};

fn main() -> io::Result<()> {
    let out = File::create("out.scad")?;
    let mut out = BufWriter::new(out);

    let x_scale = 5;
    let y_scale = 2;

    let rounded_cube = |size: Vector3, radius: f64| {
        let corner = Cylinder::with_radius(radius, size.z);
        hull![
            corner.translate((radius, radius, 0.)),
            corner.translate((size.x - radius, radius, 0.)),
            corner.translate((radius, size.y - radius, 0.)),
            corner.translate((size.x - radius, size.y - radius, 0.)),
        ]
    };

    let full_base = ((0..y_scale).flat_map(|y| {
        (0..x_scale).map(move |x| {
            (rounded_cube((37, 37, 2.7).into(), 3.)
                + hull![
                    rounded_cube((37, 37, 0.000001).into(), 3.).translate((0, 0, 2.7)),
                    rounded_cube((42, 42, 0.000001).into(), 3.).translate((-2.5, -2.5, 5)),
                ])
            .translate((2.5, 2.5, 0))
            .translate([42 * x, 42 * y, 0])
        })
    }))
    .collect::<DynUnion<_>>()
        + rounded_cube((42 * x_scale, 42 * y_scale, 1).into(), 3.).translate([0, 0, 5]);

    Scad::builder()
        .number_of_segments(50)
        .build()
        .to_scad(&mut out)?;

    full_base.to_scad(&mut out)?;

    Ok(())
}
