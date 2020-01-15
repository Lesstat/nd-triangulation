#[macro_use]
extern crate cpp;

cpp! {{
    #include <iterator>
    #include <CGAL/Epick_d.h>
    #include <CGAL/Triangulation.h>

    using DynDimension = CGAL::Dynamic_dimension_tag;
    using K = CGAL::Epick_d<DynDimension>;

    using Vertex = CGAL::Triangulation_vertex<K, uint64_t>;
    using FullCell = CGAL::Triangulation_full_cell<K, uint64_t>;
    using TDS = CGAL::Triangulation_data_structure<DynDimension, Vertex, FullCell>;
    using Triangulation = CGAL::Triangulation<K, TDS>;

}}
cpp! {{
    using Point = Triangulation::Point;
    using Facet_iterator = Triangulation::Facet_iterator;
    using Facet = Triangulation::Facet;

    using Full_cell_handle = Triangulation::Full_cell_handle;
    using Vertex_handle = Triangulation::Vertex_handle;
    using Full_cells = std::vector<Full_cell_handle>;
}}

mod cell;
mod vertex;

pub use cell::*;
pub use vertex::*;

/// Triangulation
///
/// Uses the dD triangulation package from CGAL internally.
#[derive(Debug, PartialEq, Eq)]
pub struct Triangulation {
    /// Pointer to CGAL triangulation
    ptr: *mut u8,
    /// Dimension of the triangulation
    dim: usize,
    next_point_id: usize,
}

impl Triangulation {
    /// Create new triangulation for points of size/dimension `dim`
    pub fn new(dim: usize) -> Triangulation {
        let ptr = unsafe { Self::init_triangulation_ptr(dim) };
        Triangulation {
            ptr,
            dim,
            next_point_id: 0,
        }
    }

    unsafe fn init_triangulation_ptr(dim: usize) -> *mut u8 {
        cpp!([dim as "size_t"] -> *mut u8 as "Triangulation*"{
            return new Triangulation(dim);
        })
    }

    /// Add point to the triangulation.
    ///
    /// The operation fails if `coords` has the wrong dimension.
    pub fn add_vertex(&mut self, coords: &[f64]) -> Result<usize, String> {
        if coords.len() != self.dim {
            return Err(format!(
                "Point has incorrect dimension ({} != {})",
                coords.len(),
                self.dim
            ));
        }
        let id = unsafe { self.add_vertex_internal(coords) };
        Ok(id)
    }

    unsafe fn add_vertex_internal(&mut self, coords: &[f64]) -> usize {
        let tri = self.ptr;
        let dim = self.dim;
        let coords = coords.as_ptr();
        let point_id = self.next_point_id;

        cpp!([tri as "Triangulation*", dim as "size_t", coords as "double*", point_id as "size_t"] {
            auto p = Point(dim, &coords[0], &coords[dim]);
            auto vertex = tri->insert(p);
            auto& id = vertex->data();
            id = point_id;
        });
        self.next_point_id += 1;
        point_id
    }

    /// Returns a iterator over all convex hull cells/facets.
    ///
    /// This allocates a vector with cell handles internally and is
    /// not implemented in the typical streaming fashion of rust iterators.
    pub fn convex_hull_cells(&self) -> CellIter {
        let cells = unsafe { self.gather_ch_cells() };
        CellIter::new(self, cells)
    }

    unsafe fn gather_ch_cells(&self) -> *mut u8 {
        let tri = self.ptr;
        cpp!([tri as "Triangulation*"] -> *mut u8 as "Full_cells*" {
            auto infinite_full_cells = new Full_cells();
            std::back_insert_iterator<Full_cells> out(*infinite_full_cells);
            tri->incident_full_cells(tri->infinite_vertex(), out);
            return infinite_full_cells;
        })
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
fn test_vertices_have_to_be_of_right_dimension() {
    let mut tri = Triangulation::new(3);
    assert!(tri.add_vertex(&[1.0]).is_err());
    assert!(tri.add_vertex(&[1.0, 2.0]).is_err());
    assert!(tri.add_vertex(&[1.0, 2.0, 3.0]).is_ok());
    assert!(tri.add_vertex(&[4.0, 5.0, 6.0]).is_ok());
    assert!(tri.add_vertex(&[1.0, 2.0, 3.0, 4.0]).is_err());
}

#[test]
fn test_empty_triangulation_has_pseudo_cell() {
    let tri = Triangulation::new(3);
    let ch_cells = tri.convex_hull_cells();

    assert_eq!(1, ch_cells.count());
}

#[test]
fn test_convex_hull_has_right_size() {
    let mut tri = Triangulation::new(2);

    tri.add_vertex(&[1.0, 1.0]).unwrap();
    tri.add_vertex(&[2.0, 1.0]).unwrap();
    tri.add_vertex(&[1.5, 1.5]).unwrap();

    let ch_cells = tri.convex_hull_cells();
    assert_eq!(3, ch_cells.count());
}

#[test]
fn test_convex_hull_has_right_cells() {
    let mut tri = Triangulation::new(2);

    let p1 = &[1.0, 1.0];
    let p2 = &[2.0, 1.0];
    let p3 = &[1.5, 1.5];

    let id1 = tri.add_vertex(p1).unwrap();
    let id2 = tri.add_vertex(p2).unwrap();
    let id3 = tri.add_vertex(p3).unwrap();

    let ch_cells = tri.convex_hull_cells();

    for cell in ch_cells {
        let mut all_vertices: Vec<_> = cell.vertices().collect();

        all_vertices.dedup_by_key(|p| p.id());

        assert_eq!(2, all_vertices.len());

        let only_input_vertices = all_vertices
            .iter()
            .map(Vertex::id)
            .all(|id| id == id1 || id == id2 || id == id3);
        assert!(only_input_vertices);
    }
}
