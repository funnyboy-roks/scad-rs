# Scad

> [!WARNING]
>
> This library, while generally functional, is not intended for use and
> is really just a proof-of-concept at the moment.  If you are
> interested in using this a proper library, let me know and I'll make
> it so! :)

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
    .objects(&obj)
    .build()
    .to_scad(&mut io::stdout().lock());
```

## Modelling

When modelling an object, I use the following command

```sh
./watch.sh <example>
```
