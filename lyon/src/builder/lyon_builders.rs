use crate::{builder::ShapeBuilder, shape::Vertex};
use lyon_tessellation::{
    math::Point, BasicGeometryBuilder, FillAttributes, FillGeometryBuilder, FillVertexConstructor,
    GeometryBuilder, GeometryBuilderError, StrokeAttributes, StrokeGeometryBuilder,
    StrokeVertexConstructor, VertexId,
};

impl FillVertexConstructor<Vertex> for ShapeBuilder {
    fn new_vertex(&mut self, point: Point, mut attributes: FillAttributes) -> Vertex {
        let attributes = attributes.interpolated_attributes();
        self.new_vertex(point, attributes)
    }
}

impl StrokeVertexConstructor<Vertex> for ShapeBuilder {
    fn new_vertex(&mut self, point: Point, mut attributes: StrokeAttributes) -> Vertex {
        let attributes = attributes.interpolated_attributes();
        self.new_vertex(point, attributes)
    }
}

impl FillGeometryBuilder for ShapeBuilder {
    fn add_fill_vertex(
        &mut self,
        position: Point,
        mut attributes: FillAttributes,
    ) -> Result<VertexId, GeometryBuilderError> {
        let attributes = attributes.interpolated_attributes();
        self.add_vertex(position, attributes)
    }
}

impl StrokeGeometryBuilder for ShapeBuilder {
    fn add_stroke_vertex(
        &mut self,
        position: Point,
        mut attributes: StrokeAttributes,
    ) -> Result<VertexId, GeometryBuilderError> {
        let attributes = attributes.interpolated_attributes();
        self.add_vertex(position, attributes)
    }
}

impl GeometryBuilder for ShapeBuilder {
    fn begin_geometry(&mut self) {}

    fn end_geometry(&mut self) -> lyon_tessellation::Count {
        lyon_tessellation::Count {
            vertices: self.vertices.len() as u32,
            indices: self.indicies.len() as u32,
        }
    }

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

impl BasicGeometryBuilder for ShapeBuilder {
    fn add_vertex(&mut self, position: Point) -> Result<VertexId, GeometryBuilderError> {
        let color = self.default_color;
        self.add_vertex(position, &color)
    }
}
