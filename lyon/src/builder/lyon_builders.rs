use lyon_tessellation::{
    FillGeometryBuilder, FillVertex, FillVertexConstructor, GeometryBuilder, GeometryBuilderError,
    StrokeGeometryBuilder, StrokeVertex, StrokeVertexConstructor, VertexId,
};

use crate::builder::ShapeBuilder;
use crate::shape::Vertex;

impl FillVertexConstructor<Vertex> for ShapeBuilder {
    fn new_vertex(&mut self, mut vertex: FillVertex) -> Vertex {
        let position = vertex.position();
        let attributes = vertex.interpolated_attributes();
        self.new_vertex(position, attributes)
    }
}

impl StrokeVertexConstructor<Vertex> for ShapeBuilder {
    fn new_vertex(&mut self, mut vertex: StrokeVertex) -> Vertex {
        let position = vertex.position();
        let attributes = vertex.interpolated_attributes();
        self.new_vertex(position, attributes)
    }
}

impl FillGeometryBuilder for ShapeBuilder {
    fn add_fill_vertex(
        &mut self,
        mut vertex: FillVertex,
    ) -> Result<VertexId, GeometryBuilderError> {
        let position = vertex.position();
        let attributes = vertex.interpolated_attributes();
        self.add_vertex(position, attributes)
    }
}

impl StrokeGeometryBuilder for ShapeBuilder {
    fn add_stroke_vertex(
        &mut self,
        mut vertex: StrokeVertex,
    ) -> Result<VertexId, GeometryBuilderError> {
        let position = vertex.position();
        let attributes = vertex.interpolated_attributes();
        self.add_vertex(position, attributes)
    }
}

impl GeometryBuilder for ShapeBuilder {
    fn begin_geometry(&mut self) {}

    fn end_geometry(&mut self) {}

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        self.indicies.push(a.0 as u16);
        self.indicies.push(b.0 as u16);
        self.indicies.push(c.0 as u16);
    }

    fn abort_geometry(&mut self) {
        self.vertices.clear();
        self.indicies.clear();
    }
}

// impl FillGeometryBuilder for ShapeBuilder {
//     fn add_fill_vertex(&mut self, vertex: FillVertex) -> Result<VertexId,
// GeometryBuilderError> {         let color = self.default_color;
//         self.add_vertex(vertex.position(), &color.interpolated_attributes())
//     }
// }

// // impl StrokeGeometryBuilder for ShapeBuilder {
// //     fn add_stroke_vertex(
// //         &mut self,
// //         vertex: StrokeVertex,
// //     ) -> Result<VertexId, GeometryBuilderError> {
// //         todo!()
// //     }
// // }
