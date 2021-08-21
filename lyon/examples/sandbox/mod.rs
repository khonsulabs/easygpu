use easygpu::{figures::Size, prelude::*, wgpu::TextureUsage};
use easygpu_lyon::{LyonPipeline, Srgb, VertexShaderSource};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

const MSAA_SAMPLE_COUNT: u32 = 4;

pub trait Sandbox: Sized + 'static {
    fn create(renderer: &Renderer) -> Self;
    fn pipeline(&self) -> &'_ LyonPipeline<Srgb>;
    fn render<'a, 'b>(&'a self, pass: &'b mut easygpu::wgpu::RenderPass<'a>);

    fn run() -> Result<(), easygpu::error::Error> {
        env_logger::init();
        let event_loop = EventLoop::new();
        let window = Window::new(&event_loop).unwrap();
        let size = window.inner_size();

        // Setup renderer
        let instance = easygpu::wgpu::Instance::new(easygpu::wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let mut renderer = futures::executor::block_on(Renderer::for_surface(
            surface,
            &instance,
            MSAA_SAMPLE_COUNT,
        ))?;
        let sandbox = Self::create(&renderer);
        let size = Size::new(size.width, size.height).cast::<u32>();

        let mut textures =
            renderer.swap_chain(size, PresentMode::default(), Srgb::sampler_format());

        let multisample_texture = renderer.texture(
            size,
            Srgb::sampler_format(),
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
            MSAA_SAMPLE_COUNT > 1,
        );

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    textures = renderer.swap_chain(
                        Size::new(size.width, size.height).cast::<u32>(),
                        PresentMode::default(),
                        Srgb::sampler_format(),
                    );
                    *control_flow = ControlFlow::Wait;
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
            Event::RedrawRequested(_) =>
                if let Ok(output) = textures.next_texture() {
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
                },
            _ => {
                *control_flow = ControlFlow::Wait;
            }
        });
    }
}
