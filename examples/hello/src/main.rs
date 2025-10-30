use std::{sync::Arc, time::Instant};

use anyhow::Result;
use rosemetal::*;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowId},
};

pub struct State {
    window: Arc<Window>,
    device: Arc<MTLDevice>,
    view: Arc<MTLView>,
    queue: Arc<MTLCommandQueue>,
    render_pass: Arc<MTLRenderPass>,
    render_pipeline_state: MTLRenderPipelineState,

    triangle_buffer: Arc<MTLBuffer<MTLFloat3>>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();

        let instance = RMLInstance::new(Some(RMLLayer {
            window_display: window.display_handle()?.as_raw(),
            window_handle: window.window_handle()?.as_raw(),
            width: size.width,
            height: size.height,
        }))?;

        let device = MTLDevice::create(instance)?;

        let view = MTLView::request(device.clone(), Some(MTLViewSettings { vsync: true.into() }))?;

        let queue = MTLCommandQueue::new(device.clone())?;

        let position = -2.0;

        let triangle_buffer = device.make_buffer(
            &[
                MTLFloat3::new(-position, -position, 0.0),
                MTLFloat3::new(position, -position, 0.0),
                MTLFloat3::new(0.0, position, 0.0),
            ],
            MTLBufferUsage::Vertex,
        )?;

        let library = device.new_library(&std::fs::read("Shaders.metallib")?)?;

        let vertex_function = Some(library.get_function("vertex_shader", MTLFunctionType::Vertex)?);
        let fragment_function =
            Some(library.get_function("fragment_shader", MTLFunctionType::Fragment)?);

        let render_pipeline = MTLRenderPipelineDescriptor {
            label: "Triangle Rendering Pipeline".to_string(),
            vertex_function,
            fragment_function,
            color_attachments: vec![MTLRenderPipelineColorAttachment {
                pixel_format: view.pixel_format().clone(),
            }],
            ..Default::default()
        };

        let render_pass = MTLRenderPass::new(
            device.clone(),
            MTLRenderPassDescriptor {
                color_attachments: vec![MTLRenderPassColorAttachment {
                    load_action: MTLLoadAction::Clear,
                    store_action: MTLStoreAction::Store,
                }],
                ..Default::default()
            },
        );

        let render_pipeline_state = device.new_render_pipeline_state(render_pipeline)?;

        Ok(Self {
            window,
            device,
            view,
            queue,
            render_pass,
            render_pipeline_state,

            triangle_buffer,
        })
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {}

    pub fn render(&mut self) {
        let start_time = Instant::now();

        let buffer = MTLCommandBuffer::new(self.queue.clone()).unwrap();

        let drawable = self.view.next_drawable(self.device.clone()).unwrap();

        let begin = MTLBeginRenderPassDescriptor {
            color_attachments: vec![MTLBeginRenderPassColorAttachment {
                clear_color: MTLClearColor {
                    red: 41.0 / 255.0,
                    green: 42.0 / 255.0,
                    blue: 48.0 / 255.0,
                    alpha: 1.0,
                },
                texture: drawable.clone(),
            }],
            ..Default::default()
        };

        let encoder =
            MTLRenderCommandEncoder::new(buffer.clone(), self.render_pass.clone(), begin).unwrap();

        encoder
            .set_render_pipeline_state(&self.render_pipeline_state)
            .unwrap();

        encoder.set_vertex_buffer(&self.triangle_buffer).unwrap();
        encoder
            .draw_primitives(MTLPrimitiveType::Triangle, 0, 3)
            .unwrap();

        encoder.end_encoding().unwrap();

        buffer.present(drawable);

        buffer.commit().unwrap();

        println!("{:?}", start_time.elapsed());

        self.window.request_redraw();
    }
}

pub struct App {
    state: Option<State>,
}

impl App {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut window_attributes = Window::default_attributes();

        window_attributes.title = "Hello RoseMetal!".to_string();

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.state = Some(pollster::block_on(State::new(window)).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
                state.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                state.render();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => match (code, state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                _ => {}
            },
            _ => {}
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        self.state = Some(event);
    }
}

pub fn run() -> Result<()> {
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}

fn main() {
    run().unwrap();
}
