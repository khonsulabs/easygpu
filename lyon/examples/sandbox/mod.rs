use easygpu::figures::Size;
use easygpu::prelude::*;
use easygpu::wgpu::{PresentMode, TextureUsages};
use easygpu_lyon::{LyonPipeline, Srgb, VertexShaderSource};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

const MSAA_SAMPLE_COUNT: u32 = 4;

pub trait Sandbox: Sized + 'static {
    fn create(renderer: &Renderer) -> Self;
    fn pipeline(&self) -> &'_ LyonPipeline<Srgb>;
    fn render<'a>(&'a self, pass: &mut easygpu::wgpu::RenderPass<'a>);

    fn run() -> anyhow::Result<()> {
        env_logger::init();
        let event_loop = EventLoop::new();
        let window = Window::new(&event_loop).unwrap();
        let size = window.inner_size();

        // Setup renderer
        let instance = easygpu::wgpu::Instance::new(easygpu::wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(&window) }?;
        let mut renderer = futures::executor::block_on(Renderer::for_surface(
            surface,
            &instance,
            MSAA_SAMPLE_COUNT,
        ))?;
        let sandbox = Self::create(&renderer);
        let size = Size::new(size.width, size.height).cast::<u32>();

        renderer.configure(size, PresentMode::Fifo, Srgb::sampler_format());

        let mut multisample_texture = renderer.texture(
            size,
            Srgb::sampler_format(),
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            MSAA_SAMPLE_COUNT > 1,
        );

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(new_size) => {
                    let new_size = Size::new(new_size.width, new_size.height).cast::<u32>();
                    renderer.configure(new_size, PresentMode::Fifo, Srgb::sampler_format());
                    // Recreate the texture to match the new output size.
                    multisample_texture = renderer.texture(
                        new_size,
                        Srgb::sampler_format(),
                        TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                        MSAA_SAMPLE_COUNT > 1,
                    );
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                if let Ok(output) = renderer.current_frame() {
                    let mut frame = renderer.frame();
                    renderer.update_pipeline(
                        sandbox.pipeline(),
                        ScreenTransformation::ortho(
                            0.,
                            0.,
                            output.size.width as f32,
                            output.size.height as f32,
                            -1.,
                            1.,
                        ),
                    );

                    {
                        let mut pass = frame.pass(
                            PassOp::Clear(Rgba::TRANSPARENT),
                            &output,
                            Some(&multisample_texture.view),
                        );

                        sandbox.render(&mut pass);
                    }
                    renderer.present(frame);
                }
            }
            _ => {
                *control_flow = ControlFlow::Wait;
            }
        });
    }
}
