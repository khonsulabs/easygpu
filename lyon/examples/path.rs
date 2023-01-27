use easygpu::prelude::*;
use easygpu_lyon::{LyonPipeline, Shape, ShapeBuilder, Srgb, VertexShaderSource};
use lyon_tessellation::{
    math::point,
    path::{traits::PathBuilder, Path},
    FillOptions, StrokeOptions,
};

mod sandbox;
use sandbox::Sandbox;

fn main() -> anyhow::Result<()> {
    PathExample::run()
}

struct PathExample {
    pipeline: LyonPipeline<Srgb>,
    shape: Shape,
}

impl Sandbox for PathExample {
    fn create(renderer: &Renderer) -> Self {
        let pipeline = renderer.pipeline(Blending::default(), Srgb::sampler_format());

        let mut builder = ShapeBuilder::default();

        // RGBA colors specified for each vertex
        let mut path_builder = Path::builder_with_attributes(4);
        path_builder.begin(point(50., 50.), &[1., 0., 0., 1.]);
        path_builder.line_to(point(100., 150.), &[0., 1., 0., 1.]);
        path_builder.line_to(point(150., 50.), &[0., 0., 1., 1.]);
        path_builder.close();
        let path = path_builder.build();
        builder
            .fill(&path, &FillOptions::default())
            .expect("Error tesselating path");

        // White outline
        builder.default_color = [1., 1., 1., 1.];
        let mut path_builder = Path::builder();
        path_builder.begin(point(50., 50.));
        path_builder.line_to(point(100., 150.));
        path_builder.line_to(point(150., 50.));
        path_builder.close();
        let path = path_builder.build();
        builder
            .stroke(&path, &StrokeOptions::default())
            .expect("Error tesselating path");

        let shape = builder.prepare(renderer);

        Self { pipeline, shape }
    }

    fn pipeline(&self) -> &'_ LyonPipeline<Srgb> {
        &self.pipeline
    }

    fn render<'a, 'b>(&'a self, pass: &'b mut easygpu::wgpu::RenderPass<'a>) {
        pass.set_easy_pipeline(&self.pipeline);
        self.shape.draw(pass);
    }
}
