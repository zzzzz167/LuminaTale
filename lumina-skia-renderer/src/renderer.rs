use crate::vk_utils::context::VulkanRenderContext;
use crate::vk_utils::renderer::VulkanRenderer;
use skia_safe::{Color4f, Paint, Point, Rect};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[derive(Default)]
pub struct SkiaRenderer {
    pub render_ctx: VulkanRenderContext,
    pub renderer: Option<VulkanRenderer>
}

impl SkiaRenderer {
    pub fn new() -> EventLoop<()> {
        EventLoop::new().unwrap()
    }
}

impl ApplicationHandler for SkiaRenderer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Lumina SKIA VULKAN TEST")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
                        .with_visible(true),
                )
                .unwrap(),
        );
        
        self.renderer = Some(
            self.render_ctx
                .renderer_for_window(event_loop, window.clone()),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(_) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.invalidate_swapchain();
                    renderer.window.request_redraw();
                }
            },
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.prepare_swapchain();

                    renderer.draw_and_present(|canvas, size| {
                        let canvas_size = skia_safe::Size::new(size.width, size.height);
                        canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));

                        let rect_size = canvas_size / 2.0;
                        let rect = Rect::from_point_and_size(
                            Point::new(
                                (canvas_size.width - rect_size.width) / 2.0,
                                (canvas_size.height - rect_size.height) / 2.0,
                            ),
                            rect_size,
                        );
                        canvas.draw_rect(
                            rect,
                            &Paint::new(Color4f::new(0.0, 0.0, 1.0, 1.0), None),
                        );
                    });
                }
            }
            _ => {}
        }
    }
}