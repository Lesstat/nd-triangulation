#[macro_use]
extern crate cpp;

cpp! {{
    #include <iostream>
    #include <iterator>
    #include <CGAL/Epick_d.h>
    #include <CGAL/Triangulation.h>

    using K = CGAL::Epick_d<CGAL::Dynamic_dimension_tag>;
    using Triangulation =CGAL::Triangulation<K>;

    using Point = Triangulation::Point;
    using Facet_iterator = Triangulation::Facet_iterator;
    using Facet = Triangulation::Facet;

    using Full_cell_handle = Triangulation::Full_cell_handle;
    using Vertex_handle = Triangulation::Vertex_handle;
    using Full_cells = std::vector<Full_cell_handle>;
}}

#[derive(Debug, PartialEq, Eq)]
pub struct Triangulation {
    ptr: *mut u8,
    dim: usize,
}

impl Triangulation {
    pub fn new(dim: usize) -> Triangulation {
        let ptr = unsafe { Self::init_triangulation_ptr(dim) };
        Triangulation { ptr, dim }
    }

    unsafe fn init_triangulation_ptr(dim: usize) -> *mut u8 {
        cpp!([dim as "size_t"] -> *mut u8 as "Triangulation*"{
            return new Triangulation(dim);
        })
    }

    pub fn add_point(&mut self, coords: &[f64]) -> Result<(), String> {
        if coords.len() != self.dim {
            return Err(format!(
                "Point has incorrect dimension ({} != {})",
                coords.len(),
                self.dim
            ));
        }
        unsafe {
            self.add_point_internal(coords);
        }
        Ok(())
    }

    unsafe fn add_point_internal(&mut self, coords: &[f64]) {
        let tri = self.ptr;
        let dim = self.dim;
        let coords = coords.as_ptr();

        cpp!([tri as "Triangulation*", dim as "size_t", coords as "double*"] {
            auto p = Point(dim, &coords[0], &coords[dim]);
            tri->insert(p);
        });
    }

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

#[derive(Debug)]
pub struct CellIter<'a> {
    cur: usize,
    size: usize,
    cells: *mut u8,
    tri: &'a Triangulation,
}

impl<'a> CellIter<'a> {
    fn new(tri: &'a Triangulation, cells: *mut u8) -> CellIter<'a> {
        let size = unsafe {
            cpp!([cells as "Full_cells*"] -> usize as "size_t" {
                return cells->size();
            })
        };

        CellIter {
            cur: 0,
            size,
            cells,
            tri,
        }
    }

    unsafe fn cell_ptr(&self, cur: usize) -> *mut u8 {
        let cells = self.cells;
        cpp!([cells as "Full_cells*", cur as "size_t"] -> *mut u8 as "Full_cell_handle" {
            auto& cell = (*cells)[cur];
            return cell;
        })
    }
}

impl<'a> Iterator for CellIter<'a> {
    type Item = Cell<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.size {
            return None;
        }

        let cur = self.cur;
        let ptr = unsafe { self.cell_ptr(cur) };
        self.cur += 1;
        Some(Cell { ptr, tri: self.tri })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cell<'a> {
    ptr: *mut u8,
    tri: &'a Triangulation,
}

impl<'a> Cell<'a> {
    pub fn points(&self) -> PointIter<'_> {
        PointIter {
            cur: 0,
            cell: &self,
        }
    }
}

impl<'a> Drop for CellIter<'a> {
    fn drop(&mut self) {
        let cells = self.cells;
        unsafe {
            cpp!([cells as "Full_cells*"]{
            delete cells;
                })
        }
    }
}

pub struct PointIter<'a> {
    cur: usize,
    cell: &'a Cell<'a>,
}

impl<'a> PointIter<'a> {

    #[rustfmt::skip]
    unsafe fn skip_bogus_vertices(&self) -> i64 {
        let tri = self.cell.tri.ptr;
        let cell = self.cell.ptr;
	let cur = self.cur;
        cpp!([tri as "Triangulation*", cell as "Full_cell_handle", cur as "size_t"] -> i64 as "int64_t" {
	    auto v = cell->vertices_begin();
	    std::advance(v, cur);
	    if (v == cell->vertices_end()){
	        return -1;
	    }
	    if (*v == Vertex_handle() || tri->is_infinite(*v)){
		std::advance(v,1);
		return 1;
	    }
	    return 0;

        })
    }

	#[rustfmt::skip]
    unsafe fn get_point(&mut self) -> Option<&'a [f64]> {
	let cur_update = self.skip_bogus_vertices();
	
	if cur_update < 0{
	    return None;
	}
	self.cur += cur_update as usize;
	
        let cell = self.cell.ptr;
	let cur = self.cur;
	let ptr = cpp!([cell as "Full_cell_handle", cur as "size_t"] -> *const f64 as "const double*"{

	    auto v = cell->vertices_begin();
	    std::advance(v, cur);

	    if (v != cell->vertices_end()){
		Vertex_handle vert = *v;
		auto& p = vert->point();
		return p.data();
	    }
	    return nullptr;
	    
	});

        if ptr.is_null() {
	    return None;
        }
        let slice = std::slice::from_raw_parts(ptr, self.cell.tri.dim);
        Some(slice)
    }
}

impl<'a> Iterator for PointIter<'a> {
    type Item = &'a [f64];

    fn next(&mut self) -> Option<Self::Item> {
        let next = unsafe { self.get_point() };
        self.cur += 1;
        next
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

#[test]
fn test_empty_triangulation_has_pseudo_cell() {
    let tri = Triangulation::new(3);
    let ch_cells = tri.convex_hull_cells();

    assert_eq!(1, ch_cells.count());
}

#[test]
fn test_convex_hull_has_right_size() {
    let mut tri = Triangulation::new(2);

    tri.add_point(&[1.0, 1.0]).unwrap();
    tri.add_point(&[2.0, 1.0]).unwrap();
    tri.add_point(&[1.5, 1.5]).unwrap();

    let ch_cells = tri.convex_hull_cells();
    assert_eq!(3, ch_cells.count());
}

#[test]
fn test_convex_hull_has_right_cells() {
    fn to_slice(f: &[f64]) -> &[f64] {
        f
    }
    let mut tri = Triangulation::new(2);

    let p1 = &[1.0, 1.0];
    let p2 = &[2.0, 1.0];
    let p3 = &[1.5, 1.5];

    tri.add_point(p1).unwrap();
    tri.add_point(p2).unwrap();
    tri.add_point(p3).unwrap();

    let ch_cells = tri.convex_hull_cells();

    for cell in ch_cells {
        let all_points: Vec<&[f64]> = cell.points().collect();

        assert_eq!(2, all_points.len());
        let mut point_count = 0;
        if all_points.contains(&to_slice(p1)) {
            point_count += 1;
        }
        if all_points.contains(&to_slice(p2)) {
            point_count += 1;
        }
        if all_points.contains(&to_slice(p3)) {
            point_count += 1;
        }

        assert_eq!(2, point_count);
    }
}
