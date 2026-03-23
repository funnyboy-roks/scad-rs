# Scad

This is a library for writing [OpenSCAD] code using Rust structures and
type system.

This is largely just a proof-of-concept at the moment, but it does seem
to be nicer to work with than OpenSCAD directly.

Check out the examples for some demonstrations!

[OpenSCAD]: https://openscad.org/

## Usage

```rust
let obj = Cube::with_size((5, 5, 2.5)) + Sphere::with_radius(0.5).translate((0, 0, 2));

let scad = Scad::builder()
    .number_of_segments(100)
    .objects(Box::new(obj))
    .build()
    .to_scad(&mut file);
```

## Modelling

When modelling an object, I use the following command (each running in a
tmux window)

```sh
openscad --viewall out.scad # open openscad
cargo watch -w examples -w src -- cargo r --example swatch # cargo-watch to autmatically re-run when changes are made
```

and I have the example write to the `out.scad` file.
