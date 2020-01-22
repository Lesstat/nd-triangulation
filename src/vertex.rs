use crate::Cell;

/// A vertex which is part of a triangulation
pub struct Vertex<'a> {
    ptr: *mut u8, //c++ type: Vertex_handle
    cell: &'a Cell<'a>,
}

impl<'a> Vertex<'a> {
    /// Unique id between all vertices within the same triangulation
    pub fn id(&self) -> usize {
        unsafe { self.retrieve_id() }
    }

    /// Coordinates of the vertex
    pub fn coords(&self) -> &'a [f64] {
        unsafe { self.retrieve_coords() }
    }

    unsafe fn retrieve_id(&self) -> usize {
        let ptr = self.ptr;
        #[cfg(not(feature = "docs-rs"))]
        return cpp!([ptr as "Vertex_handle"] -> usize as "size_t"{
            return ptr->data();
        });

        #[cfg(feature = "docs-rs")]
        0
    }

    unsafe fn retrieve_coords(&self) -> &'a [f64] {
        let ptr = self.ptr;
        #[cfg(not(feature = "docs-rs"))]
        let point = cpp!([ptr as "Vertex_handle"] -> *const f64 as "const double*"{
            auto& p = ptr->point();
            return p.data();
        });

        #[cfg(feature = "docs-rs")]
        let point = std::ptr::null();

        std::slice::from_raw_parts(point, self.cell.tri().dim)
    }
}

/// Iterator over vertices beloning to a cell
pub struct VertexIter<'a> {
    cur: usize,
    cell: &'a Cell<'a>,
}

impl<'a> VertexIter<'a> {
    /// Creates an Iterator over the vertices of a cell
    pub fn new(cell: &'a Cell) -> VertexIter<'a> {
        VertexIter { cur: 0, cell }
    }

    #[rustfmt::skip]
    unsafe fn skip_bogus_vertices(&self) -> i64 {
        let tri = self.cell.tri().ptr;
        let cell = self.cell.ptr();
        let cur = self.cur;
        #[cfg(not(feature = "docs-rs"))]
        return cpp!([tri as "Triangulation*", cell as "Full_cell_handle", cur as "size_t"] -> i64 as "int64_t" {
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

        });

	#[cfg(feature = "docs-rs")]
	0
    }

    #[rustfmt::skip]
    unsafe fn get_vertex(&mut self) -> Option<Vertex<'a>> {
        let cur_update = self.skip_bogus_vertices();

        if cur_update < 0 {
            return None;
        }

        self.cur += cur_update as usize;

        let cell = self.cell.ptr();
        let cur = self.cur;

        #[cfg(not(feature = "docs-rs"))]
        let ptr = cpp!([cell as "Full_cell_handle", cur as "size_t"] -> *mut u8 as "Vertex_handle"{
            auto v = cell->vertices_begin();
            std::advance(v, cur);

            if (v != cell->vertices_end()){
		return *v;
            }
            return nullptr;

        });

        #[cfg(feature = "docs-rs")]
	let ptr: *mut u8 = std::ptr::null_mut();

        if ptr.is_null() {
	    return None;
        }

        Some(Vertex {
            ptr,
            cell: self.cell,
        })
    }
}

impl<'a> Iterator for VertexIter<'a> {
    type Item = Vertex<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = unsafe { self.get_vertex() };
        self.cur += 1;
        next
    }
}
