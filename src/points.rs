use crate::Cell;

pub struct Point<'a> {
    id: usize,
    coords: &'a [f64],
}

impl<'a> Point<'a> {
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn coords(&self) -> &'a [f64] {
        self.coords
    }
}

/// Iterator over points beloning to a cell
pub struct PointIter<'a> {
    cur: usize,
    cell: &'a Cell<'a>,
}

impl<'a> PointIter<'a> {
    pub fn new(cell: &'a Cell) -> PointIter<'a> {
        PointIter { cur: 0, cell }
    }

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

    unsafe fn get_point(&mut self) -> Option<Point<'a>> {
        let cur_update = self.skip_bogus_vertices();

        if cur_update < 0 {
            return None;
        }

        self.cur += cur_update as usize;

        let cell = self.cell.ptr;
        let cur = self.cur;
        let ptr = cpp!([cell as "Full_cell_handle", cur as "size_t"] -> *mut u8 as "Vertex_handle"{
            auto v = cell->vertices_begin();
            std::advance(v, cur);

            if (v != cell->vertices_end()){
        return *v;
            }
            return nullptr;

        });

        if ptr.is_null() {
            return None;
        }

        let id = cpp!([ptr as "Vertex_handle"] -> usize as "size_t"{
            return ptr->data();
        });
        let point = cpp!([ptr as "Vertex_handle"] -> *const f64 as "const double*"{
            auto& p = ptr->point();
            return p.data();
        });

        let coords = std::slice::from_raw_parts(point, self.cell.tri.dim);
        Some(Point { id, coords })
    }
}

impl<'a> Iterator for PointIter<'a> {
    type Item = Point<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = unsafe { self.get_point() };
        self.cur += 1;
        next
    }
}
