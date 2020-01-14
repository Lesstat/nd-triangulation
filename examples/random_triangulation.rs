use nd_triangulation::*;
use std::env::args;

fn main() {
    let args: Vec<_> = args().collect();
    let dim = args[1].parse().expect("Could not parse dimension");
    let point_count: usize = args[2].parse().expect("Could not parse number of points");

    let points: Vec<f64> = (0..point_count * dim).map(|_| rand::random()).collect();

    println!(
        "Creating triangulation with {} points in dimension {}",
        point_count, dim
    );

    let mut tri = Triangulation::new(dim);

    points.chunks(dim).for_each(|p| {
        tri.add_point(&p).unwrap();
    });

    println!(
        "Convex hull of triangulation consists of {} cells",
        tri.convex_hull_cells().count()
    );
}
