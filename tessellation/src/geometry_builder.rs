//! Tools to help with generating vertex and index buffers.
//!
//! ## Overview
//!
//! While it would be possible for the tessellation algorithms to manually generate vertex
//! and index buffers with a certain layout, it would mean that most code using the tessellators
//! would need to copy and convert all generated vertices in order to have their own vertex
//! layout, or even several vertex layouts, which is a very common use-case.
//!
//! In order to provide flexibility with the generation of geometry, this module provides with
//! the [`GeometryBuilder`](trait.GeometryBuilder.html) and its extension the
//! [`BezierGeometryBuilder`](trait.BezierGeometryBuilder.html) trait. The former exposes
//! the methods to facilitate adding vertices and triangles. The latter adds a method to
//! specifically handle quadratic bezier curves. Quadratic bézier curves have interesting
//! properties that make them a lot easier to render than most types of curves and we want
//! to have the option to handle them separately in the renderer.
//!
//! See the [Rendering curves](https://github.com/nical/lyon/wiki/Experiments#rendering-curves)
//! section in the project's wiki for more details about the advantages of handling quadratic
//! bézier curves separately in the tessellator and the renderer.
//!
//! This modules provides with a basic implementation of these traits through the following types:
//!
//! * The struct [`VertexBuffers<T>`](struct.VertexBuffers.html) is a simple pair of vectors of u32
//!   indices and T (generic parameter) vertices.
//! * The struct [`BuffersBuilder`](struct.BuffersBuilder.html) which implements
//!   [`BezierGeometryBuilder`](trait.BezierGeometryBuilder.html) and writes into a
//!   [`VertexBuffers`](struct.VertexBuffers.html).
//! * The trait [`VertexConstructor`](trait.VertexConstructor.html) used by
//!   [`BuffersBuilder`](struct.BuffersBuilder.html) in order to generate any vertex type. In the
//!   example below, a struct `WithColor` implements the `VertexConstructor` trait in order to
//!   create vertices composed of a 2d position and a color value from an input 2d position.
//!   This separates the construction of vertex values from the assembly of the vertex buffers.
//!   Another, simpler example of vertex constructor is the [`Identity`](struct.Identity.html)
//!   constructor which just returns its input, untransformed.
//!   `VertexConstructor<Input, Output>` is implemented for all closures `Fn(Input) -> Output`.
//!
//! Geometry builders are a practical way to add one last step to the tessellation pipeline,
//! such as applying a transform or clipping the geometry.
//!
//! While this is module designed to facilitate the generation of vertex buffers and index
//! buffers, nothing prevents a given GeometryBuilder implementation to only generate a
//! vertex buffer without indices, or write into a completely different format.
//! These builder traits are at the end of the tessellation pipelines and are meant for
//! users of this crate to be able to adapt the output of the tessellators to their own
//! needs.
//!
//! ## Examples
//!
//! ### Generating custom vertices
//!
//! The example below implements the `VertexConstructor` trait in order to use a custom
//! vertex type `MyVertex` (containing position and color), storing the tessellation in a
//! `VertexBuffers<MyVertex, u16>`, and tessellates two shapes with different colors.
//!
//! ```
//! extern crate lyon_tessellation as tess;
//! use tess::{VertexConstructor, VertexBuffers, BuffersBuilder, FillVertex, FillOptions};
//! use tess::basic_shapes::fill_circle;
//! use tess::math::{Point, point};
//!
//! // Our custom vertex.
//! #[derive(Copy, Clone, Debug)]
//! pub struct MyVertex {
//!   position: [f32; 2],
//!   color: [f32; 4],
//! }
//!
//! // The vertex constructor. This is the object that will be used to create the custom
//! // verticex from the information provided by the tessellators.
//! struct WithColor([f32; 4]);
//!
//! impl VertexConstructor<Point, MyVertex> for WithColor {
//!     fn new_vertex(&mut self, vertex: Point) -> MyVertex {
//!         MyVertex {
//!             position: [vertex.x, vertex.y],
//!             color: self.0,
//!         }
//!     }
//! }
//!
//! fn main() {
//!     let mut output: VertexBuffers<MyVertex, u16> = VertexBuffers::new();
//!     // Tessellate a red and a green circle.
//!     fill_circle(
//!         point(0.0, 0.0),
//!         10.0,
//!         &FillOptions::tolerance(0.05),
//!         &mut BuffersBuilder::new(
//!             &mut output,
//!             WithColor([1.0, 0.0, 0.0, 1.0])
//!         ),
//!     );
//!     fill_circle(
//!         point(10.0, 0.0),
//!         5.0,
//!         &FillOptions::tolerance(0.05),
//!         &mut BuffersBuilder::new(
//!             &mut output,
//!             WithColor([0.0, 1.0, 0.0, 1.0])
//!         ),
//!     );
//!
//!     println!(" -- {} vertices, {} indices", output.vertices.len(), output.indices.len());
//! }
//! ```
//!
//! ### Generating a completely custom output
//!
//! Using `VertexBuffers<T>` is convenient and probably fits a lot of use cases, but
//! what if we do not want to write the geometry in a pair of vectors?
//! Perhaps we want to write the geometry in a different data structure or directly
//! into gpu-accessible buffers mapped on the CPU?
//!
//! ```
//! extern crate lyon_tessellation as tess;
//! use tess::{GeometryBuilder, StrokeGeometryBuilder, StrokeOptions, StrokeVertex, Count};
//! use tess::geometry_builder::{VertexId, GeometryBuilderError, VertexSource};
//! use tess::basic_shapes::stroke_polyline;
//! use tess::math::{Point, point};
//! use std::fmt::Debug;
//! use std::u32;
//!
//! // A geometry builder that writes the result of the tessellation to stdout instead
//! // of filling vertex and index buffers.
//! pub struct ToStdOut {
//!     vertices: u32,
//!     indices: u32,
//! }
//!
//! impl ToStdOut {
//!      pub fn new() -> Self { ToStdOut { vertices: 0, indices: 0 } }
//! }
//!
//! // This one takes any vertex type that implements Debug, so it will work with both
//! // FillVertex and StrokeVertex.
//! impl GeometryBuilder for ToStdOut {
//!     fn begin_geometry(&mut self) {
//!         // Reset the vertex in index counters.
//!         self.vertices = 0;
//!         self.indices = 0;
//!         println!(" -- begin geometry");
//!     }
//!
//!     fn end_geometry(&mut self) -> Count {
//!         println!(" -- end geometry, {} vertices, {} indices", self.vertices, self.indices);
//!         Count {
//!             vertices: self.vertices,
//!             indices: self.indices,
//!         }
//!     }
//!
//!     fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
//!         println!("triangle ({}, {}, {})", a.offset(), b.offset(), c.offset());
//!         self.indices += 3;
//!     }
//!
//!     fn abort_geometry(&mut self) {
//!         println!(" -- oops!");
//!     }
//! }
//!
//! impl StrokeGeometryBuilder for ToStdOut {
//!     fn add_stroke_vertex(&mut self, vertex: StrokeVertex) -> Result<VertexId, GeometryBuilderError> {
//!         println!("vertex {:?}", vertex.position);
//!         if self.vertices >= u32::MAX {
//!             return Err(GeometryBuilderError::TooManyVertices);
//!         }
//!         self.vertices += 1;
//!         Ok(VertexId(self.vertices as u32 - 1))
//!     }
//! }
//!
//! fn main() {
//!     let mut output = ToStdOut::new();
//!     stroke_polyline(
//!         [point(0.0, 0.0), point(10.0, 0.0), point(5.0, 5.0)].iter().cloned(),
//!         true,
//!         &StrokeOptions::default(),
//!         &mut output,
//!     );
//! }
//! ```
//!
//! ### Writing a tessellator
//!
//! The example below is the implementation of `basic_shapes::fill_rectangle`.
//!
//! ```
//! use lyon_tessellation::geometry_builder::*;
//! use lyon_tessellation::{FillVertex, TessellationResult};
//! use lyon_tessellation::math::{Rect, vector, point};
//!
//! // A tessellator that generates an axis-aligned quad.
//! // Returns a structure containing the number of vertices and number of indices allocated
//! // during the execution of this method.
//! pub fn fill_rectangle<Output>(rect: &Rect, output: &mut Output) -> TessellationResult
//! where
//!     Output: FillGeometryBuilder
//! {
//!     output.begin_geometry();
//!     // The FillGeometryBuilder interface provides the ability to specify
//!     // where vertices come from. To keep this example simple we pass a
//!     // dummy "no-source" implementation.
//!     let mut src = NoSource;
//!     // Create the vertices...
//!     let min = rect.min();
//!     let max = rect.max();
//!     let a = output.add_fill_vertex(min, &mut src)?;
//!     let b = output.add_fill_vertex(point(max.x, min.y), &mut src)?;
//!     let c = output.add_fill_vertex(max, &mut src)?;
//!     let d = output.add_fill_vertex(point(min.x, max.y), &mut src)?;
//!     // ...and create triangle form these points. a, b, c, and d are relative offsets in the
//!     // vertex buffer.
//!     output.add_triangle(a, b, c);
//!     output.add_triangle(a, c, d);
//!
//!     Ok(output.end_geometry())
//! }
//! ```

pub use crate::path::{VertexId, EndpointId, EventId, Index};
use crate::math::Point;
use crate::StrokeVertex;

use std::marker::PhantomData;
use std::ops::Add;
use std::convert::From;
use std;

/// An error that can happen while generating geometry.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GeometryBuilderError {
    InvalidVertex,
    TooManyVertices,
}

#[derive(Clone, Debug)]
pub enum VertexSource {
    Endpoint { id: EndpointId },
    Edge { edge: EventId, from: EndpointId, to: EndpointId, t: f32 },
}

pub struct NoSource;
impl Iterator for NoSource {
    type Item = VertexSource;
    fn next(&mut self) -> Option<VertexSource> { None }
}

/// An interface separating tessellators and other geometry generation algorithms from the
/// actual vertex construction.
///
/// See the [`geometry_builder`](index.html) module documentation for more detailed explanation.
pub trait GeometryBuilder {
    /// Called at the beginning of a generation.
    ///
    /// end_geometry must be called before begin_geometry is called again.
    fn begin_geometry(&mut self);

    /// Called at the end of a generation.
    /// Returns the number of vertices and indices added since the last time begin_geometry was
    /// called.
    fn end_geometry(&mut self) -> Count;

    /// Insert a triangle made of vertices that were added after the last call to begin_geometry.
    ///
    /// This method can only be called between begin_geometry and end_geometry.
    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId);

    /// abort_geometry is called instead of end_geometry if an error occurred while producing
    /// the geometry and we won't be able to finish.
    ///
    /// The implementation is expected to discard the geometry that was generated since the last
    /// time begin_geometry was called, and to remain in a usable state.
    fn abort_geometry(&mut self);
}

pub trait FillGeometryBuilder: GeometryBuilder {
    /// Inserts a vertex, providing its position, and optionally a normal.
    /// Returns a vertex id that is only valid between begin_geometry and end_geometry.
    ///
    /// This method can only be called between begin_geometry and end_geometry.
    fn add_fill_vertex(&mut self, vertex: Point, src: &mut dyn Iterator<Item=VertexSource>) -> Result<VertexId, GeometryBuilderError>;
}

pub trait StrokeGeometryBuilder: GeometryBuilder {
    /// Inserts a vertex, providing its position, and optionally a normal.
    /// Returns a vertex id that is only valid between begin_geometry and end_geometry.
    ///
    /// This method can only be called between begin_geometry and end_geometry.
    fn add_stroke_vertex(&mut self, vertex: StrokeVertex) -> Result<VertexId, GeometryBuilderError>;
}


/// An interface with similar goals to `GeometryBuilder` for algorithms that pre-build
/// the vertex and index buffers.
///
/// This is primarily intended for efficient interaction with the libtess2 tessellator
/// from the `lyon_tess2` crate.
pub trait GeometryReceiver<Vertex> {

    fn set_geometry(
        &mut self,
        vertices: &[Vertex],
        indices: &[u32]
    );
}

/// Structure that holds the vertex and index data.
///
/// Usually written into though temporary `BuffersBuilder` objects.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct VertexBuffers<VertexType, IndexType> {
    pub vertices: Vec<VertexType>,
    pub indices: Vec<IndexType>,
}

impl<VertexType, IndexType> VertexBuffers<VertexType, IndexType> {
    /// Constructor
    pub fn new() -> Self { VertexBuffers::with_capacity(512, 1024) }

    /// Constructor
    pub fn with_capacity(num_vertices: usize, num_indices: usize) -> Self {
        VertexBuffers {
            vertices: Vec::with_capacity(num_vertices),
            indices: Vec::with_capacity(num_indices),
        }
    }
}

/// A temporary view on a `VertexBuffers` object which facilitate the population of vertex and index
/// data.
///
/// `BuffersBuilders` record the vertex offset from when they are created so that algorithms using
/// them don't need to worry about offsetting indices if some geometry was added beforehand. This
/// means that from the point of view of a `BuffersBuilder` user, the first added vertex is at always
/// offset at the offset 0 and `VertexBuilder` takes care of translating indices adequately.
///
/// Often, algorithms are built to generate vertex positions without knowledge of eventual other
/// vertex attributes. The `VertexConstructor` does the translation from generic `Input` to `VertexType`.
/// If your logic generates the actual vertex type directly, you can use the `SimpleBuffersBuilder`
/// convenience typedef.
pub struct BuffersBuilder<'l, VertexType: 'l, IndexType:'l, Input, Ctor> {
    buffers: &'l mut VertexBuffers<VertexType, IndexType>,
    vertex_offset: Index,
    index_offset: Index,
    vertex_constructor: Ctor,
    _marker: PhantomData<Input>,
}

impl<'l, VertexType: 'l, IndexType:'l, Input, Ctor> BuffersBuilder<'l, VertexType, IndexType, Input, Ctor> {
    pub fn new(
        buffers: &'l mut VertexBuffers<VertexType, IndexType>,
        ctor: Ctor,
    ) -> Self {
        let vertex_offset = buffers.vertices.len() as Index;
        let index_offset = buffers.indices.len() as Index;
        BuffersBuilder {
            buffers,
            vertex_offset,
            index_offset,
            vertex_constructor: ctor,
            _marker: PhantomData,
        }
    }

    pub fn buffers<'a, 'b: 'a>(&'b self) -> &'a VertexBuffers<VertexType, IndexType> {
        self.buffers
    }
}

/// Creates a `BuffersBuilder`.
pub fn vertex_builder<VertexType, IndexType, Input, Ctor>(
    buffers: &mut VertexBuffers<VertexType, IndexType>,
    ctor: Ctor,
) -> BuffersBuilder<VertexType, IndexType, Input, Ctor>
where
    Ctor: VertexConstructor<Input, VertexType>
{
    BuffersBuilder::new(buffers, ctor)
}

/// A trait specifying how to create vertex values.
pub trait VertexConstructor<Input, VertexType> {
    fn new_vertex(&mut self, input: Input) -> VertexType;
}

/// A dummy vertex constructor that just forwards its inputs.
pub struct Identity;
impl<T> VertexConstructor<T, T> for Identity {
    fn new_vertex(&mut self, input: T) -> T { input }
}

impl<F, Input, VertexType> VertexConstructor<Input, VertexType> for F
    where F: Fn(Input) -> VertexType
{
    fn new_vertex(&mut self, vertex: Input) -> VertexType {
        self(vertex)
    }
}

/// A `BuffersBuilder` that takes the actual vertex type as input.
pub type SimpleBuffersBuilder<'l, VertexType> = BuffersBuilder<'l, VertexType, u16, VertexType, Identity>;

/// Creates a `SimpleBuffersBuilder`.
pub fn simple_builder<VertexType>(buffers: &mut VertexBuffers<VertexType, u16>)
    -> SimpleBuffersBuilder<VertexType> {
    let vertex_offset = buffers.vertices.len() as Index;
    let index_offset = buffers.indices.len() as Index;
    BuffersBuilder {
        buffers,
        vertex_offset,
        index_offset,
        vertex_constructor: Identity,
        _marker: PhantomData,
    }
}

/// Number of vertices and indices added during the tessellation.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Count {
    pub vertices: u32,
    pub indices: u32,
}

impl Add for Count {
    type Output = Count;
    fn add(self, other: Count) -> Count {
        Count {
            vertices: self.vertices + other.vertices,
            indices: self.indices + other.indices,
        }
    }
}

impl<'l, VertexType, IndexType, Vertex, Ctor> GeometryBuilder
    for BuffersBuilder<'l, VertexType, IndexType, Vertex, Ctor>
where
    VertexType: 'l + Clone,
    IndexType: Add + From<VertexId> + MaxIndex,
{
    fn begin_geometry(&mut self) {
        self.vertex_offset = self.buffers.vertices.len() as Index;
        self.index_offset = self.buffers.indices.len() as Index;
    }

    fn end_geometry(&mut self) -> Count {
        Count {
            vertices: self.buffers.vertices.len() as u32 - self.vertex_offset,
            indices: self.buffers.indices.len() as u32 - self.index_offset,
        }
    }

    fn abort_geometry(&mut self) {
        self.buffers.vertices.truncate(self.vertex_offset as usize);
        self.buffers.indices.truncate(self.index_offset as usize);
    }

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        debug_assert!(a != b);
        debug_assert!(a != c);
        debug_assert!(b != c);
        debug_assert!(a != VertexId::INVALID);
        debug_assert!(b != VertexId::INVALID);
        debug_assert!(c != VertexId::INVALID);
        self.buffers.indices.push((a + self.vertex_offset).into());
        self.buffers.indices.push((b + self.vertex_offset).into());
        self.buffers.indices.push((c + self.vertex_offset).into());
    }
}

impl<'l, VertexType, IndexType, Ctor> FillGeometryBuilder
    for BuffersBuilder<'l, VertexType, IndexType, Point, Ctor>
where
    VertexType: 'l + Clone,
    IndexType: Add + From<VertexId> + MaxIndex,
    Ctor: VertexConstructor<Point, VertexType>,
{
    fn add_fill_vertex(&mut self, v: Point, _src: &mut dyn Iterator<Item=VertexSource>) -> Result<VertexId, GeometryBuilderError> {
        self.buffers.vertices.push(self.vertex_constructor.new_vertex(v));
        let len = self.buffers.vertices.len();
        if len > IndexType::max_index() {
            return Err(GeometryBuilderError::TooManyVertices);
        }
        Ok(VertexId((len - 1) as Index - self.vertex_offset))
    }
}

impl<'l, VertexType, IndexType, Ctor> StrokeGeometryBuilder
    for BuffersBuilder<'l, VertexType, IndexType, StrokeVertex, Ctor>
where
    VertexType: 'l + Clone,
    IndexType: Add + From<VertexId> + MaxIndex,
    Ctor: VertexConstructor<StrokeVertex, VertexType>,
{
    fn add_stroke_vertex(&mut self, v: StrokeVertex) -> Result<VertexId, GeometryBuilderError> {
        self.buffers.vertices.push(self.vertex_constructor.new_vertex(v));
        let len = self.buffers.vertices.len();
        if len > IndexType::max_index() {
            return Err(GeometryBuilderError::TooManyVertices);
        }
        Ok(VertexId((len - 1) as Index - self.vertex_offset))
    }
}

impl<'l, VertexType, IndexType, InputVertex, Ctor> GeometryReceiver<InputVertex>
    for BuffersBuilder<'l, VertexType, IndexType, InputVertex, Ctor>
where
    VertexType: 'l + Clone,
    IndexType: From<VertexId>,
    Ctor: VertexConstructor<InputVertex, VertexType>,
    InputVertex: Clone,
{
    fn set_geometry(
        &mut self,
        vertices: &[InputVertex],
        indices: &[u32]
    ) {
        for v in vertices {
            let vertex = self.vertex_constructor.new_vertex(v.clone());
            self.buffers.vertices.push(vertex);
        }
        for idx in indices {
            self.buffers.indices.push(IndexType::from(idx.clone().into()));
        }
    }
}

/// A geometry builder that does not output any geometry.
///
/// Mostly useful for testing.
pub struct NoOutput {
    count: Count,
}

impl NoOutput {
    pub fn new() -> Self {
        NoOutput { count: Count { vertices: 0, indices: 0 } }
    }
}

impl GeometryBuilder for NoOutput {
    fn begin_geometry(&mut self) {
        self.count.vertices = 0;
        self.count.indices = 0;
    }

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        debug_assert!(a != b);
        debug_assert!(a != c);
        debug_assert!(b != c);
        self.count.indices += 3;
    }

    fn end_geometry(&mut self) -> Count { self.count }

    fn abort_geometry(&mut self) {}
}

impl FillGeometryBuilder for NoOutput {
    fn add_fill_vertex(&mut self, _: Point, _src: &mut dyn Iterator<Item=VertexSource>) -> Result<VertexId, GeometryBuilderError> {
        if self.count.vertices >= std::u32::MAX {
            return Err(GeometryBuilderError::TooManyVertices);
        }
        self.count.vertices += 1;
        Ok(VertexId(self.count.vertices as Index - 1))
    }
}

impl StrokeGeometryBuilder for NoOutput {
    fn add_stroke_vertex(&mut self, _: StrokeVertex) -> Result<VertexId, GeometryBuilderError> {
        if self.count.vertices >= std::u32::MAX {
            return Err(GeometryBuilderError::TooManyVertices);
        }
        self.count.vertices += 1;
        Ok(VertexId(self.count.vertices as Index - 1))
    }
}

impl<V> GeometryReceiver<V> for NoOutput {
    fn set_geometry(&mut self, _vertices: &[V], _indices: &[u32]) {}
}

/// Provides the maximum value of an index.
///
/// This should be the maximum value representable by the index type up
/// to u32::MAX because the tessellators can't internally represent more
/// than u32::MAX indices.
pub trait MaxIndex {
    fn max_index() -> usize;
}

impl MaxIndex for u8 { fn max_index() -> usize { std::u8::MAX as usize } }
impl MaxIndex for i8 { fn max_index() -> usize { std::i8::MAX as usize } }
impl MaxIndex for u16 { fn max_index() -> usize { std::u16::MAX as usize } }
impl MaxIndex for i16 { fn max_index() -> usize { std::i16::MAX as usize } }
impl MaxIndex for u32 { fn max_index() -> usize { std::u32::MAX as usize } }
impl MaxIndex for i32 { fn max_index() -> usize { std::i32::MAX as usize } }
// The tessellators internally use u32 indices so we can't have more than u32::MAX
impl MaxIndex for u64 { fn max_index() -> usize { std::u32::MAX as usize } }
impl MaxIndex for i64 { fn max_index() -> usize { std::u32::MAX as usize } }
impl MaxIndex for usize { fn max_index() -> usize { std::u32::MAX as usize } }
impl MaxIndex for isize { fn max_index() -> usize { std::u32::MAX as usize } }

#[test]
fn test_closure() {
    use crate::math::{Point, point, vector};

    let translation = vector(1.0, 0.0);

    let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();

    {
        // A builder that just translates all vertices by `translation`.
        let mut builder = vertex_builder(&mut buffers, |position| {
            position + translation
        });

        builder.begin_geometry();
        let mut src = NoSource;
        let a = builder.add_fill_vertex(point(0.0, 0.0), &mut src).unwrap();
        let b = builder.add_fill_vertex(point(1.0, 0.0), &mut src).unwrap();
        let c = builder.add_fill_vertex(point(1.0, 1.0), &mut src).unwrap();
        let d = builder.add_fill_vertex(point(0.0, 1.0), &mut src).unwrap();
        builder.add_triangle(a, b, c);
        builder.add_triangle(a, c, d);
        builder.end_geometry();
    }

    assert_eq!(buffers.vertices, vec![
        point(1.0, 0.0),
        point(2.0, 0.0),
        point(2.0, 1.0),
        point(1.0, 1.0),
    ]);
}
