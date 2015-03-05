use std::cmp::PartialEq;

pub type Index = u16;

#[derive(Copy, Clone, Show, PartialEq, Eq)]
pub struct Id<T> { pub handle: Index }
#[derive(Copy, Clone, Show, PartialEq, Eq)]
pub struct Vertex_;
#[derive(Copy, Clone, Show, PartialEq, Eq)]
pub struct Edge_;
#[derive(Copy, Clone, Show, PartialEq, Eq)]
pub struct Face_;

pub type VertexId = Id<Vertex_>;
pub type EdgeId = Id<Edge_>;
pub type FaceId = Id<Face_>;

impl EdgeId {
    pub fn is_valid(self) -> bool { self.handle != 0 }
    pub fn as_index(self) -> usize { self.handle as usize - 1 }
}

impl FaceId {
    pub fn is_valid(self) -> bool { self.handle != 0 }
    pub fn as_index(self) -> usize { self.handle as usize - 1 }
}

impl VertexId {
    pub fn is_valid(self) -> bool { self.handle != 0 }
    pub fn as_index(self) -> usize { self.handle as usize - 1 }
}

pub fn no_edge() -> EdgeId { EdgeId { handle: 0 } }

pub fn no_face() -> FaceId { FaceId { handle: 0 } }

pub fn no_vertex() -> VertexId { VertexId { handle: 0 } }

pub fn edge_id(index: Index) -> EdgeId { EdgeId { handle: index + 1 } }

pub fn face_id(index: Index) -> FaceId { FaceId { handle: index + 1 } }

pub fn vertex_id(index: Index) -> VertexId { VertexId { handle: index + 1 } }

#[derive(Copy, Clone, Show, PartialEq)]
pub struct IdRange<T> {
    pub first: Id<T>,
    pub count: Index,
}

impl<T: Copy> IdRange<T> {
    pub fn iter(&self) -> IdRangeIterator<T> {
        return IdRangeIterator { range: *self };
    }
}

pub type VertexIdRange = IdRange<Vertex_>;
pub type EdgeIdRange = IdRange<Edge_>;
pub type FaceIdRange = IdRange<Face_>;

#[derive(Copy, Clone, Show, PartialEq)]
pub struct HalfEdge {
    pub next: EdgeId, // next HalfEdge around the face
    pub prev: EdgeId, // previous HalfEdge around the face
    pub vertex: VertexId, // vertex this edge origins from
    pub opposite: EdgeId,
    pub face: FaceId,
}

#[derive(Copy, Clone, Show, PartialEq)]
pub struct Face {
    pub first_edge: EdgeId,
    pub first_interrior: FaceId,
    pub next_sibling: FaceId,
}

#[derive(Copy, Clone, Show, PartialEq)]
pub struct Vertex {
    pub first_edge: EdgeId,
}

pub struct ConnectivityKernel {
    edges: Vec<HalfEdge>,
    vertices: Vec<Vertex>,
    faces: Vec<Face>
}

impl ConnectivityKernel {

    pub fn vertex(&self, id: VertexId) -> &Vertex {
        assert!(id.is_valid());
        &self.vertices[id.handle as usize - 1]
    }

    fn vertex_mut(&mut self, id: VertexId) -> &mut Vertex {
        assert!(id.is_valid());
        &mut self.vertices[id.handle as usize - 1]
    }
    
    pub fn face(&self, id: FaceId) -> &Face {
        assert!(id.is_valid());
        &self.faces[id.handle as usize - 1]
    }

    fn face_mut(&mut self, id: FaceId) -> &mut Face {
        assert!(id.is_valid());
        &mut self.faces[id.handle as usize - 1]
    }
    
    pub fn edge(&self, id: EdgeId) -> &HalfEdge {
        assert!(id.is_valid());
        &self.edges[id.handle as usize - 1]
    }

    fn edge_mut(&mut self, id: EdgeId) -> &mut HalfEdge {
        assert!(id.is_valid());
        &mut self.edges[id.handle as usize - 1]
    }

    pub fn edges(&self) -> &[HalfEdge] { &self.edges[] }

    pub fn faces(&self) -> &[Face] { &self.faces[] }

    pub fn vertices(&self) -> &[Vertex] { &self.vertices[] }

    pub fn first_edge(&self) -> EdgeId { edge_id(0) }

    pub fn first_face(&self) -> FaceId { FaceId { handle: 1 } }

    pub fn first_vertex(&self) -> VertexId { VertexId { handle: 1 } }

    pub fn vertex_ids(&self) -> VertexIdIterator {
        VertexIdIterator {
            current: 1,
            stop: self.vertices.len() as Index + 1,
        }
    }

    pub fn edge_ids(&self) -> EdgeIdIterator {
        EdgeIdIterator {
            current: 1,
            stop: self.edges.len() as Index + 1,
        }
    }

    pub fn face_ids(&self) -> FaceIdIterator {
        FaceIdIterator {
            current: 1,
            stop: self.faces.len() as Index + 1,
        }
    }

    pub fn walk_edges_around_face<'l>(&'l self, id: FaceId) -> FaceEdgeIterator<'l> {
        let edge = self.face(id).first_edge;
        let prev = self.edge(edge).prev;
        return FaceEdgeIterator {
            kernel: self,
            current_edge: edge,
            last_edge: prev,
            done: false,
        }
    }

    pub fn walk_edges_around_face_reverse<'l>(&'l self, id: FaceId) -> ReverseFaceEdgeIterator<'l> {
        let edge = self.face(id).first_edge;
        return ReverseFaceEdgeIterator {
            kernel: self,
            current_edge: edge,
            last_edge: self.edge(edge).next,
            done: false,
        }
    }

    pub fn next_edge_around_vertex(&self, id: EdgeId) -> EdgeId {
        return self.edge(self.edge(id).opposite).next;
    }

    pub fn assert_edge_invariants(&self, id: EdgeId) {
        assert_eq!(self.edge(self.edge(id).opposite).opposite, id);
        assert_eq!(self.edge(self.edge(id).next).prev, id);
        assert_eq!(self.edge(self.edge(id).prev).next, id);
        assert_eq!(
            self.edge(id).vertex,
            self.edge(self.edge(self.edge(id).opposite).next).vertex
        );
        assert_eq!(self.edge(id).face, self.edge(self.edge(id).next).face);
        assert_eq!(self.edge(id).face, self.edge(self.edge(id).prev).face);
    }

    pub fn assert_face_invariants(&self, face: FaceId) {
        for e in self.walk_edges_around_face(face) {
            println!("assert edge {:?}", e);
            self.assert_edge_invariants(e);
            //assert_eq!(self.edge(e).face, face);
        }
    }

    /// Insert a vertex on this edge and return the id of the new vertex
    pub fn split_edge(&mut self, id: EdgeId) -> VertexId {
        // from:
        //     a ---[id]----------------------------------------> b
        //     a <----------------------------------[opposite]--- b
        // to:
        //     a ---[id]------------> new_vertex ---[new_edge]--> b
        //     a <--[new_opposite]--- new_vertex <--[opposite]--- b

        let new_vertex = VertexId { handle: self.vertices.len() as Index + 1 };
        let new_edge = edge_id(self.vertices.len() as Index);
        let new_opposite = edge_id(self.vertices.len() as Index + 1);

        self.vertices.push(Vertex {
            first_edge: new_edge,
        });

        // new_edge
        let edge = *self.edge(id);
        self.edges.push(HalfEdge {
            vertex: edge.vertex,
            opposite: edge.opposite,
            face: edge.face,
            next: edge.next,
            prev: id,
        });

        // new_opposite
        let opposite = *self.edge(edge.opposite);
        self.edges.push(HalfEdge {
            vertex: opposite.vertex,
            opposite: id,
            face: opposite.face,
            next: opposite.next,
            prev: edge.opposite,
        });

        // patch up existing edges
        self.edge_mut(id).vertex = new_vertex;
        self.edge_mut(id).next = new_edge;
        self.edge_mut(edge.opposite).vertex = new_vertex;
        self.edge_mut(edge.opposite).next = new_opposite;

        return new_vertex;
    }

    /// Split a face in two along the vertices that a_prev and b_prev point to
    pub fn split_face(&mut self, a_next: EdgeId, b_next: EdgeId) -> FaceId {
        //
        // a_prev--> va -a_next->
        //          | ^
        //   f1     n |
        //          | |
        //          | o     f2
        //          v |
        // <-b_next- vb <--b_prev
        // ______________________
        //
        // f1: original_face
        // f2: new_face
        // n: new_edge
        // o: new_opposite_edge


        println!(" ++++ split vertices {} {} edge {} {}",
            self.edge(a_next).vertex.as_index(), self.edge(b_next).vertex.as_index(),
            a_next.as_index(), b_next.as_index()
        );

        let a_prev = self.edge(a_next).prev;
        let b_prev = self.edge(b_next).prev;

        let original_face = self.edge(a_prev).face;

        println!(" precondition ");
        self.assert_face_invariants(original_face);

        assert_eq!(original_face, self.edge(b_prev).face);
        assert!(self.edge(a_next).next != b_next);
        assert!(a_prev != b_next);

        let va = self.edge(a_next).vertex;
        let vb = self.edge(b_next).vertex;
        let new_edge = edge_id(self.edges.len() as Index); // va -> vb
        let new_opposite_edge = edge_id(self.edges.len() as Index + 1); // vb -> va

        self.faces.push(Face {
            first_edge: new_opposite_edge,
            first_interrior: no_face(),
            next_sibling: no_face(),
        });

        let new_face = face_id(self.faces.len() as Index - 1);

        // new_edge
        self.edges.push(HalfEdge {
            next: b_next,
            prev: a_prev,
            opposite: new_opposite_edge,
            face: original_face,
            vertex: va,
        });

        // new_opposite_edge
        self.edges.push(HalfEdge {
            next: a_next,
            prev: b_prev,
            opposite: new_edge,
            face: new_face,
            vertex: vb,
        });

        self.edge_mut(a_prev).next = new_edge;
        self.edge_mut(a_next).prev = new_opposite_edge;
        self.edge_mut(b_prev).next = new_opposite_edge;
        self.edge_mut(b_next).prev = new_edge;
        self.face_mut(original_face).first_edge = new_edge;

        let mut it = new_opposite_edge;
        loop {
            let edge = &mut self.edge_mut(it);
            edge.face = new_face;
            it = edge.next;
            if it == new_opposite_edge { break; }
        }

        println!("original_face");
        self.assert_face_invariants(original_face);

        println!("new_face");
        self.assert_face_invariants(new_face);

        return new_face;
    }

    pub fn join_vertices(&mut self, _: VertexId, _: VertexId) {
        panic!("not implemented");
    }

    /// Merge b into a (removing b)
    pub fn merge_vertices(&mut self, _: VertexId, _: VertexId) {
        panic!("not implemented");
    }

    pub fn extrude_edge(&mut self, _: EdgeId) {
        panic!("not implemented");
    }

    pub fn extrude_face(&mut self, _: FaceId, _face_per_edge: bool) {
        panic!("not implemented");
    }

    pub fn count_edges_around_face(&self, face: FaceId) -> u32 {
        let face = self.face(face);
        let stop = self.edge(face.first_edge).prev;
        let mut it = face.first_edge;
        let mut count: u32 = 1;
        loop {
            if it == stop { break; }
            count += 1;
            it = self.edge(it).next;
            if count > 10 { panic!(); }
        }
        return count;
    }

    fn add_loop(
        &mut self,
        n_vertices: Index,
        face1: FaceId,
        face2: FaceId,
        is_hole: bool
    ) -> (VertexIdRange, EdgeIdRange) {
        assert!(face1 != face2);
        let edge_offset = self.edges.len() as Index;
        let vertex_offset = self.vertices.len() as Index;
        for i in (0 .. n_vertices) {
            self.vertices.push(Vertex { first_edge: edge_id(edge_offset + i) });
            self.edges.push(HalfEdge {
                vertex: vertex_id(vertex_offset + i),
                opposite: edge_id(edge_offset + n_vertices * 2 - i - 1),
                face: face1,
                next: edge_id(edge_offset + modulo(i as i32 + 1, n_vertices as i32) as Index),
                prev: edge_id(edge_offset + modulo(i as i32 - 1, n_vertices as i32) as Index),
            });
        }

        for i in (0 .. n_vertices) {
            let inv_i = n_vertices - i - 1;
            self.edges.push(HalfEdge {
                vertex: vertex_id(vertex_offset + (inv_i + 1)%n_vertices),
                opposite: edge_id(edge_offset + inv_i),
                face: face2,
                next: edge_id(edge_offset + n_vertices + modulo(i as i32 + 1, n_vertices as i32) as Index),
                prev: edge_id(edge_offset + n_vertices + modulo(i as i32 - 1, n_vertices as i32) as Index),
            });
        }
        self.face_mut(face1).first_edge = edge_id(edge_offset);
        if !is_hole {
            self.face_mut(face2).first_edge = edge_id(edge_offset + n_vertices);
        }

        self.assert_face_invariants(face1);
        self.assert_face_invariants(face2);

        return (
            IdRange { first: vertex_id(self.vertices.len() as Index - n_vertices - 1), count: n_vertices },
            IdRange { first: edge_id(self.edges.len() as Index - 2*n_vertices - 1), count: 2*n_vertices },
        );
    }

    /// constructor
    pub fn from_loop(n_vertices: Index) -> ConnectivityKernel {
        assert!(n_vertices >= 3);
        let main_face = face_id(0);
        let back_face = face_id(1);
        let mut kernel = ConnectivityKernel {
            faces: vec!(
                Face {
                    first_edge: no_edge(), // set in add_loop
                    first_interrior: no_face(),
                    next_sibling: no_face(),
                },
                Face {
                    first_edge: no_edge(), // set in add_loop
                    first_interrior: no_face(),
                    next_sibling: no_face(),
                }
            ),
            vertices: vec!(),
            edges: vec!(),
        };
        kernel.add_loop(n_vertices, main_face, back_face, false);
        assert!(kernel.face(main_face).first_edge.handle != 0);
        assert!(kernel.face(back_face).first_edge.handle != 0);
        return kernel;
    }

    pub fn add_hole(&mut self, face: FaceId, n_vertices: Index) -> (FaceId, VertexIdRange, EdgeIdRange) {
        let new_face = face_id(self.faces.len() as Index);

        let sibling = self.face(face).first_interrior;
        self.face_mut(face).first_interrior = new_face;

        self.faces.push(Face {
            first_edge: no_edge(),
            first_interrior: no_face(),
            next_sibling: sibling,
        });

        let (new_vertices, new_edges) = self.add_loop(n_vertices, new_face, face, true);

        return (new_face, new_vertices, new_edges);
    }
}

/// Iterates over the half edges around a face.
pub struct FaceEdgeIterator<'l> {
    kernel: &'l ConnectivityKernel,
    current_edge: EdgeId,
    last_edge: EdgeId,
    done: bool,
}

impl<'l> Iterator for FaceEdgeIterator<'l> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<EdgeId> {
        let res = self.current_edge;
        if self.done {
            return None;
        }
        if self.current_edge == self.last_edge {
            self.done = true;
        }
        self.current_edge = self.kernel.edge(self.current_edge).next;
        return Some(res);
    }
}

/// Iterates over the half edges around a face in reverse order.
pub struct ReverseFaceEdgeIterator<'l> {
    kernel: &'l ConnectivityKernel,
    current_edge: EdgeId,
    last_edge: EdgeId,
    done: bool,
}

impl<'l> Iterator for ReverseFaceEdgeIterator<'l> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<EdgeId> {
        let res = self.current_edge;
        if self.done {
            return None;
        }
        if self.current_edge == self.last_edge {
            self.done = true;
        }
        self.current_edge = self.kernel.edge(self.current_edge).prev;
        return Some(res);
    }
}

/// Iterates over the half edges that point to a vertex.
pub struct VertexEdgeIterator<'l> {
    kernel: &'l ConnectivityKernel,
    current_edge: EdgeId,
    first_edge: EdgeId,
}

impl<'l> Iterator for VertexEdgeIterator<'l> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<EdgeId> {
        if !self.current_edge.is_valid() {
            return None;
        }
        let temp = self.current_edge;
        self.current_edge = self.kernel.edge(self.kernel.edge(self.current_edge).next).opposite;
        if self.current_edge == self.first_edge {
            self.current_edge = no_edge();
        }
        return Some(temp);
    }
}

pub struct VertexIdIterator<'l> {
    current: Index,
    stop: Index,
}

impl<'l> Iterator for VertexIdIterator<'l> {
    type Item = VertexId;

    fn next(&mut self) -> Option<VertexId> {
        if self.current == self.stop { return None; }
        self.current += 1;
        return Some(VertexId { handle: self.current - 1 });
    }
}

pub struct EdgeIdIterator<'l> {
    current: Index,
    stop: Index,
}

impl<'l> Iterator for EdgeIdIterator<'l> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<EdgeId> {
        if self.current == self.stop { return None; }
        self.current += 1;
        return Some(EdgeId { handle: self.current - 1 });
    }
}

pub struct FaceIdIterator<'l> {
    current: Index,
    stop: Index,
}

impl<'l> Iterator for FaceIdIterator<'l> {
    type Item = FaceId;

    fn next(&mut self) -> Option<FaceId> {
        if self.current == self.stop { return None; }
        self.current += 1;
        return Some(FaceId { handle: self.current - 1 });
    }
}

#[derive(Copy, Clone, Show, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

impl Direction {
    pub fn reverse(self) -> Direction {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
        }
    }
}

#[derive(Copy, Clone)]
pub struct EdgeCirculator<'l> {
    kernel: &'l ConnectivityKernel,
    edge: EdgeId,
}

impl<'l> EdgeCirculator<'l> {
    pub fn new(kernel: &'l ConnectivityKernel, edge: EdgeId) -> EdgeCirculator{
        EdgeCirculator {
            kernel: kernel,
            edge: edge,
        }
    }

    pub fn edge(&'l self) -> &'l HalfEdge { self.kernel.edge(self.edge) }

    pub fn next(self) -> EdgeCirculator<'l> {
        EdgeCirculator {
            kernel: self.kernel,
            edge: self.edge().next,
        }
    }

    pub fn prev(self) -> EdgeCirculator<'l> {
        EdgeCirculator {
            kernel: self.kernel,
            edge: self.edge().prev,
        }
    }

    pub fn advance(self, direction: Direction) -> EdgeCirculator<'l> {
        match direction {
            Direction::Forward => self.next(),
            Direction::Backward => self.prev(),
        }
    }

    pub fn edge_id(&self) -> EdgeId { self.edge }

    pub fn vertex_id(&self) -> VertexId { self.edge().vertex }

    pub fn face_id(&self) -> FaceId { self.edge().face }
}

impl<'l> PartialEq<EdgeCirculator<'l>> for EdgeCirculator<'l> {
    fn eq(&self, other: &EdgeCirculator) -> bool {
        return self.edge.eq(&other.edge);
    }
    fn ne(&self, other: &EdgeCirculator) -> bool {
        return self.edge.ne(&other.edge);
    }
}

#[derive(Copy, Clone)]
pub struct DirectedEdgeCirculator<'l> {
    circulator: EdgeCirculator<'l>,
    direction: Direction,
}

impl<'l> DirectedEdgeCirculator<'l> {
    pub fn new(kernel: &'l ConnectivityKernel, edge: EdgeId, direction: Direction) -> DirectedEdgeCirculator {
        DirectedEdgeCirculator {
            circulator: EdgeCirculator::new(kernel, edge),
            direction: direction,
        }
    }

    pub fn edge(&'l self) -> &'l HalfEdge { self.circulator.edge() }

    pub fn next(self) -> DirectedEdgeCirculator<'l> {
        DirectedEdgeCirculator {
            circulator: self.circulator.advance(self.direction),
            direction: self.direction,
        }
    }

    pub fn prev(self) -> DirectedEdgeCirculator<'l> {
        DirectedEdgeCirculator {
            circulator: self.circulator.advance(self.direction.reverse()),
            direction: self.direction,
        }
    }

    pub fn advance(self, direction: Direction) -> DirectedEdgeCirculator<'l> {
        match self.direction == direction {
            true => self.next(),
            false => self.prev(),
        }
    }

    pub fn edge_id(&self) -> EdgeId { self.circulator.edge }

    pub fn vertex_id(&self) -> VertexId { self.circulator.vertex_id() }

    pub fn face_id(&self) -> FaceId { self.circulator.face_id() }

    pub fn direction(&self) -> Direction { self.direction }

    pub fn set_direction(&mut self, direction: Direction) { self.direction = direction; }
}

impl<'l> PartialEq<DirectedEdgeCirculator<'l>> for DirectedEdgeCirculator<'l> {
    fn eq(&self, other: &DirectedEdgeCirculator) -> bool {
        return self.circulator.edge.eq(&other.circulator.edge);
    }
    fn ne(&self, other: &DirectedEdgeCirculator) -> bool {
        return self.circulator.edge.ne(&other.circulator.edge);
    }
}

#[derive(Clone)]
pub struct IdRangeIterator<T> {
    range: IdRange<T>
}

impl<T:Copy> Iterator for IdRangeIterator<T> {
    type Item = IdRange<T>;
    fn next(&mut self) -> Option<IdRange<T>> {
        if self.range.count == 0 {
            return None;
        }
        let res = self.range;
        self.range.count -= 1;
        self.range.first.handle += 1;
        return Some(res);
    }
}

/// A modulo that behaves properly with negative values.
fn modulo(v: i32, m: i32) -> i32 { (v%m+m)%m }

#[test]
fn test_from_loop() {
    for n in range(3, 10) {
        let kernel = ConnectivityKernel::from_loop(n);
        let face = kernel.first_face();

        assert_eq!(kernel.count_edges_around_face(face) as Index, n);

        let mut i = 0;
        for e in kernel.walk_edges_around_face(face) {
            assert!((e.handle as usize - 1) < kernel.edges.len());
            assert_eq!(
                kernel.edge(e).vertex,
                kernel.edge(kernel.edge(kernel.edge(e).opposite).next).vertex
            );
            i += 1;
        }
        assert_eq!(i, n);

        for e in kernel.edge_ids() {
            assert_eq!(kernel.edge(kernel.edge(e).opposite).opposite, e);
            assert_eq!(kernel.edge(kernel.edge(e).next).prev, e);
            assert_eq!(kernel.edge(kernel.edge(e).prev).next, e);
        }

        let mut i = 0;
        for e in kernel.walk_edges_around_face_reverse(face) {
            assert!((e.handle as usize - 1) < kernel.edges.len());
            assert_eq!(kernel.edge(e).face, face);
            i += 1;
        }

        let face2 = kernel.edge(kernel.edge(kernel.face(face).first_edge).opposite).face;
        let mut i = 0;
        for e in kernel.walk_edges_around_face_reverse(face2) {
            assert!((e.handle as usize - 1) < kernel.edges.len());
            assert_eq!(kernel.edge(e).face, face2);
            i += 1;
        }

        assert!(face2 != face);
        assert_eq!(i, n);
    }
}

#[test]
fn test_split_face_1() {
    let mut kernel = ConnectivityKernel::from_loop(4);
    let f1 = kernel.first_face();
    let e1 = kernel.face(f1).first_edge;
    let e2 = kernel.edge(e1).next;
    let e3 = kernel.edge(e2).next;
    let e4 = kernel.edge(e3).next;
    assert_eq!(kernel.edge(e4).next, e1);
    assert_eq!(kernel.count_edges_around_face(f1), 4);

    // x---e1---->x
    // ^          |
    // |          |
    // |          e2
    // e4   f1    |
    // |          |
    // |          v
    // x<-----e3--x

    let f2 = kernel.split_face(e3, e1);

    // x---e1---->x
    // ^ \ ^   f1 |
    // | e5 \     |
    // |   \ \    e2
    // e4   \ \   |
    // |     \ e6 |
    // | f2   v \ v
    // x<-----e3--x

    assert!(f1 != f2);
    assert!(kernel.face(f1).first_edge != kernel.face(f2).first_edge);

    assert_eq!(kernel.edge(kernel.face(f1).first_edge).face, f1);
    assert_eq!(kernel.edge(kernel.face(f2).first_edge).face, f2);

    let e5 = kernel.edge(e4).next;
    let e6 = kernel.edge(e2).next;

    assert_eq!(kernel.edge(e6).next, e1);
    assert_eq!(kernel.edge(e1).prev, e6);
    assert_eq!(kernel.edge(e5).next, e3);
    assert_eq!(kernel.edge(e3).prev, e5);
    assert_eq!(kernel.edge(e6).prev, e2);
    assert_eq!(kernel.edge(e2).next, e6);
    assert_eq!(kernel.edge(e5).prev, e4);
    assert_eq!(kernel.edge(e4).next, e5);

    assert_eq!(kernel.edge(e1).face, f1);
    assert_eq!(kernel.edge(e2).face, f1);
    assert_eq!(kernel.edge(e6).face, f1);
    assert_eq!(kernel.edge(e3).face, f2);
    assert_eq!(kernel.edge(e4).face, f2);
    assert_eq!(kernel.edge(e5).face, f2);

    assert_eq!(kernel.count_edges_around_face(f1), 3);
    assert_eq!(kernel.count_edges_around_face(f2), 3);
}

#[test]
fn test_split_face_2() {
    let mut kernel = ConnectivityKernel::from_loop(10);
    let f1 = kernel.first_face();

    let e1 = kernel.face(f1).first_edge;
    let e2 = kernel.edge(e1).next;
    let e3 = kernel.edge(e2).next;
    let e4 = kernel.edge(e3).next;
    let e5 = kernel.edge(e4).next;

    let f2 = kernel.split_face(e4, e2);

    for e in kernel.walk_edges_around_face(f2) {
        assert_eq!(kernel.edge(e).face, f2);
    }

    for e in kernel.walk_edges_around_face(f1) {
        assert_eq!(kernel.edge(e).face, f1);
    }

    for dir in [Direction::Forward, Direction::Backward].iter() {
        for face in [f1, f2].iter() {
            let mut it = DirectedEdgeCirculator::new(&kernel, kernel.face(*face).first_edge, *dir);
            let stop = it.prev();
            loop {
                assert_eq!(it.face_id(), *face);
                if it == stop {
                    break;
                }
                it = it.next();
            }
        }
    }
}
