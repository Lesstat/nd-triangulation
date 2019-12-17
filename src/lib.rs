#[macro_use]
extern crate cpp;

cpp! {{
    #include <iostream>
    #include <CGAL/Epick_d.h>
    #include <CGAL/Triangulation.h>

    using K = CGAL::Epick_d<CGAL::Dynamic_dimension_tag>;
    using Triangulation =CGAL::Triangulation<K>;

    using Point = Triangulation::Point;
    using Facet_iterator = Triangulation::Facet_iterator;
    using Facet = Triangulation::Facet;
}}

pub struct Triangulation {
    ptr: *mut u8,
    dim: usize,
}

impl Triangulation {
    pub fn new(dim: usize) -> Triangulation {
        let ptr = unsafe {
            cpp!([dim as "size_t"] -> *mut u8 as "Triangulation*"{
            return new Triangulation(dim);
                })
        };
        Triangulation { ptr, dim }
    }

    pub fn add_point(&mut self, coords: &[f64]) -> Result<(), String> {
        let dim = self.dim;
        let tri = self.ptr;

        if coords.len() != dim {
            return Err(format!(
                "Point has incorrect dimension ({} != {})",
                coords.len(),
                dim
            ));
        }
        let coords = coords.as_ptr();

        unsafe {
            cpp!([tri as "Triangulation*", dim as "size_t", coords as "double*"]{

                auto p = Point(dim, &coords[0], &coords[dim]);

            tri->insert(p);
            });
        }

        Ok(())
    }
}

impl Drop for Triangulation {
    fn drop(&mut self) {
        let ptr = self.ptr;
        unsafe {
            cpp!([ptr as "Triangulation*"] {
                delete ptr;
            })
        }
    }
}

#[test]
fn test_triangulation_can_be_created_and_dropped_safely() {
    let tri = Triangulation::new(3);
    assert_eq!(3, tri.dim);
}

#[test]
fn test_points_have_to_be_of_right_dimension() {
    let mut tri = Triangulation::new(3);
    assert!(tri.add_point(&[1.0]).is_err());
    assert!(tri.add_point(&[1.0, 2.0]).is_err());
    assert!(tri.add_point(&[1.0, 2.0, 3.0]).is_ok());
    assert!(tri.add_point(&[4.0, 5.0, 6.0]).is_ok());
    assert!(tri.add_point(&[1.0, 2.0, 3.0, 4.0]).is_err());
}
