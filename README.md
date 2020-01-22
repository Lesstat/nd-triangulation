# Arbitrary Dimensional Triangulations

This library provides an interface to the [CGAL
Library](https://cgal.org) for creating Triangulations and traverse
them in arbitrary dimension. Its feature set is pretty limited, as it
is written for a very specific research purpose. However, I am willing
to look into integrating new features and of course accept pull requests.

## Example
A triangulation can be created incrementally, by adding vertices to it:

``` rust
    let mut tri = Triangulation::new(3); // We are doing a 3 dimensional triangulation

    // Everything that can be referenced as a slice can be added to the Triangulation
    tri.add_vertex(&[1.0, 1.0, 1.0]).unwrap(); // Add point doesn't work for points in the wrong dimension
    tri.add_vertex(&[2.0, 4.1, -2.3]).unwrap();
    tri.add_vertex(&[44.2, 45.4, 12.6]).unwrap();
    tri.add_vertex(&[-23.2, 24.7, 17.9]).unwrap();
```

Afterwards, we can iterate over all convex hull cells and their respective points:

``` rust
    for ch_cell in tri.convex_hull_cells() {
        for p in ch_cell.vertices() {
            println!("{:?} is on the boundary of the convex hull", p);
        }
    }
```

## Dependencies

This crate uses the cpp crate for the interaction with c++ and
specifically cgal. This means that in order to use and compile this
crate you need to have g++ as well as cgal and eigen3 installed.

## Limitations

At the current state this crate is pretty feature minimal. It offers
functionality that I need in my research and compiles in the
environment that I need (specifically archlinux right now). However, I
am willing to improve this situation if other people want to use this
and need other features or environments. Just open an issue with your
use case. Furthermore, I am no expert in rust-c++-interop. Therefore,
I can and will not promise you that there are no memory leaks or
undefined behavior. I am doing my best though ;).



