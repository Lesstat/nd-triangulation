#[macro_use]
extern crate cpp;

#[cfg(not(feature = "docs-rs"))]
cpp! {{
    #include <iterator>
    #include <CGAL/Epick_d.h>
    #include <CGAL/Triangulation.h>

    using DynDimension = CGAL::Dynamic_dimension_tag;
    using K = CGAL::Epick_d<DynDimension>;

    using Vertex = CGAL::Triangulation_vertex<K, size_t>;
    using FullCell = CGAL::Triangulation_full_cell<K, size_t>;
    using TDS = CGAL::Triangulation_data_structure<DynDimension, Vertex, FullCell>;
    using Triangulation = CGAL::Triangulation<K, TDS>;

}}

#[cfg(not(feature = "docs-rs"))]
cpp! {{
    using Point = Triangulation::Point;
    using Facet_iterator = Triangulation::Facet_iterator;
    using Facet = Triangulation::Facet;

    using Full_cell_handle = Triangulation::Full_cell_handle;
    using Vertex_handle = Triangulation::Vertex_handle;
    using Full_cells = std::vector<Full_cell_handle>;
}}

use std::fmt;

mod cell;
mod vertex;

pub use cell::*;
pub use vertex::*;

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
/// Error Type for Triangulation Errors
pub enum TriangulationError {
    /// Returned if a vertex of the wrong dimension is added.
    WrongDimension {
        actual_dim: usize,
        expected_dim: usize,
    },
}

impl fmt::Display for TriangulationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TriangulationError::*;
        match self {
            WrongDimension {
                actual_dim,
                expected_dim,
            } => write!(
                f,
                "The vertex could not be added because its dimension was {} instead of {} ",
                actual_dim, expected_dim
            ),
        }
    }
}

impl std::error::Error for TriangulationError {}

/// Triangulation
///
/// Uses the dD triangulation package from CGAL internally.
#[derive(Debug, PartialEq, Eq)]
pub struct Triangulation {
    /// Pointer to CGAL triangulation
    ptr: *mut u8, //c++ type: Triangulation*
    /// Dimension of the triangulation
    dim: usize,
    next_vertex_id: usize,
    next_cell_id: usize,
}

impl Triangulation {
    /// Create new triangulation for vertices of size/dimension `dim`
    pub fn new(dim: usize) -> Triangulation {
        let ptr = unsafe { Self::init_triangulation_ptr(dim) };
        Triangulation {
            ptr,
            dim,
            next_vertex_id: 0,
            next_cell_id: 1,
        }
    }

    unsafe fn init_triangulation_ptr(dim: usize) -> *mut u8 {
        #[cfg(not(feature = "docs-rs"))]
        return cpp!([dim as "size_t"] -> *mut u8 as "Triangulation*"{
            return new Triangulation(dim);
        });

        #[cfg(feature = "docs-rs")]
        std::ptr::null_mut()
    }

    /// Add vertex to the triangulation.
    ///
    /// The operation fails if `coords` has the wrong dimension.
    pub fn add_vertex(&mut self, coords: &[f64]) -> Result<usize, TriangulationError> {
        if coords.len() != self.dim {
            return Err(TriangulationError::WrongDimension {
                actual_dim: coords.len(),
                expected_dim: self.dim,
            });
        }
        let id = unsafe { self.add_vertex_unchecked(coords) };
        Ok(id)
    }

    /// Add vertex to triangulation without veryfing the dimension
    ///
    /// # Safety
    /// If the dimension of `coords` is too small undefined behavior might be triggered on the c++ side.
    /// If the dimension of `coords` is too large only the first `dim` values will be considered.
    pub unsafe fn add_vertex_unchecked(&mut self, coords: &[f64]) -> usize {
        let tri = self.ptr;
        let dim = self.dim;
        let coords = coords.as_ptr();
        let vertex_id = self.next_vertex_id;

        #[cfg(not(feature = "docs-rs"))]
        cpp!([tri as "Triangulation*", dim as "size_t", coords as "double*", vertex_id as "size_t"] {
            auto p = Point(dim, &coords[0], &coords[dim]);
            auto vertex = tri->insert(p);
            auto& id = vertex->data();
            id = vertex_id;
        });
        self.next_vertex_id += 1;
        vertex_id
    }

    /// Returns a iterator over all convex hull cells/facets.
    ///
    /// This allocates a vector with cell handles internally and is
    /// not implemented in the typical streaming fashion of rust iterators.
    pub fn convex_hull_cells(&mut self) -> CellIter {
        let cells = unsafe { self.gather_ch_cells() };
        CellIter::new(self, cells)
    }

    #[rustfmt::skip]
    unsafe fn gather_ch_cells(&mut self) -> *mut u8 {
        let tri = self.ptr;
        let cell_id = &mut self.next_cell_id;

        #[cfg(not(feature = "docs-rs"))]
	return cpp!([tri as "Triangulation*", cell_id as "size_t*"] -> *mut u8 as "Full_cells*" {
	    auto infinite_full_cells = new Full_cells();
	    std::back_insert_iterator<Full_cells> out(*infinite_full_cells);
	    tri->incident_full_cells(tri->infinite_vertex(), out);
	    for (auto& cell: *infinite_full_cells){
		auto& id = cell->data();
		if(id == 0){
		    id = *cell_id;
		    (*cell_id)++;
		}
	    }
	    return infinite_full_cells;
        });

	#[cfg(feature = "docs-rs")]
	std::ptr::null_mut()
    }

    /// Returns a iterator over all cells/facets of the triangulation.
    ///
    /// This allocates a vector with cell handles internally and is
    /// not implemented in the typical streaming fashion of rust iterators.
    pub fn cells(&mut self) -> CellIter {
        let cells = unsafe { self.gather_all_cells() };
        CellIter::new(self, cells)
    }

    #[rustfmt::skip]
    unsafe fn gather_all_cells(&mut self) -> *mut u8 {
        let tri = self.ptr;
        let cell_id = &mut self.next_cell_id;

        #[cfg(not(feature = "docs-rs"))]
	return cpp!([tri as "Triangulation*", cell_id as "size_t*"] -> *mut u8 as "Full_cells*" {
	    auto full_cells = new Full_cells();
	    std::back_insert_iterator<Full_cells> out(*full_cells);
	    for (auto cit = tri->full_cells_begin(); cit != tri->full_cells_end(); ++cit){
		auto cell = cit;
		auto& id = cell->data();
		if(id == 0){
		    id = *cell_id;
		    (*cell_id)++;
		}
		full_cells->push_back(cell);
	    }
	    return full_cells;
        });

	#[cfg(feature = "docs-rs")]
	std::ptr::null_mut()
    }
}

impl Drop for Triangulation {
    fn drop(&mut self) {
        let ptr = self.ptr;
        unsafe {
            #[cfg(not(feature = "docs-rs"))]
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
    assert_eq!(
        tri.add_vertex(&[1.0, 2.0, 3.0, 4.0]),
        Err(TriangulationError::WrongDimension {
            actual_dim: 4,
            expected_dim: 3
        })
    );
}

#[test]
fn test_empty_triangulation_has_pseudo_cell() {
    let mut tri = Triangulation::new(3);
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

#[test]
fn test_triangulation_has_right_size() {
    let mut tri = Triangulation::new(2);

    tri.add_vertex(&[1.0, 1.0]).unwrap();
    tri.add_vertex(&[2.0, 1.0]).unwrap();
    tri.add_vertex(&[1.5, 1.5]).unwrap();

    let cells = tri.cells();
    assert_eq!(4, cells.count());
}
