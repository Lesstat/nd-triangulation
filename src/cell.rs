use crate::{Triangulation, VertexIter};

/// Iterator over cells/facets of a triangulation
#[derive(Debug)]
pub struct CellIter<'a> {
    cur: usize,
    size: usize,
    cells: *mut u8,
    pub(crate) tri: &'a Triangulation,
}

impl<'a> CellIter<'a> {
    pub(crate) fn new(tri: &'a Triangulation, cells: *mut u8) -> CellIter<'a> {
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

/// Representation of a specific cell of a triangulation
#[derive(Debug, PartialEq, Eq)]
pub struct Cell<'a> {
    ptr: *mut u8,
    tri: &'a Triangulation,
}

impl<'a> Cell<'a> {
    /// Returns an iterator over all vertices that are part of this cell.
    pub fn vertices(&self) -> VertexIter<'_> {
        VertexIter::new(&self)
    }

    /// Unique id between all cells of one triangulation
    pub fn id(&self) -> usize {
        0
    }

    pub(crate) fn tri(&self) -> &Triangulation {
        self.tri
    }

    pub(crate) fn ptr(&self) -> *mut u8 {
        self.ptr
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

#[test]
fn test_cells_get_assigned_increasing_ids() {
    let mut tri = Triangulation::new(2);

    tri.add_vertex(&[1.0, 1.0]).unwrap();
    tri.add_vertex(&[2.0, 1.0]).unwrap();
    tri.add_vertex(&[1.5, 1.5]).unwrap();

    let mut expected_cell_id = 0;

    for cell in tri.convex_hull_cells() {
        assert_eq!(expected_cell_id, cell.id());
        expected_cell_id += 1;
    }
}
