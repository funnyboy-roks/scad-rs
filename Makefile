%.scad: examples/%.rs
	cargo r --example $* > $@

%.stl: %.scad
	openscad $^ -o $@
