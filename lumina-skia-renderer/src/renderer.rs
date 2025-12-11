use crate::painter::Painter;
use crate::vk_utils::context::VulkanRenderContext;
use crate::vk_utils::renderer::VulkanRenderer;

use winit::{
    event_loop::{ControlFlow, EventLoop, ActiveEventLoop},
    application::ApplicationHandler,
    window::{Window, WindowId},
    event::{WindowEvent, KeyEvent,ElementState},
    keyboard::{PhysicalKey, KeyCode}
};
use std::sync::Arc;

use viviscript_core::ast::Script;
use lumina_core::{Ctx, OutputEvent};
use lumina_core::event::InputEvent;
use lumina_core::renderer::driver::ExecutorHandle;

pub struct SkiaRenderer {
    render_ctx: VulkanRenderContext,
    renderer: Option<VulkanRenderer>,
    painter: Painter,
    ctx: Ctx,
    driver: Option<ExecutorHandle>,
    init_script: Option<Script>,
}

impl SkiaRenderer {
    pub fn new(script: Script) -> Self {
        Self {
            render_ctx: VulkanRenderContext::default(),
            renderer: None,
            painter: Painter::new(),
            ctx: Ctx::default(),
            driver: None,
            init_script: Some(script),
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).unwrap();
    }
}

impl ApplicationHandler for SkiaRenderer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes().with_title("LuminaTale Skia")).unwrap());
        self.renderer = Some(self.render_ctx.renderer_for_window(event_loop, window.clone()));

        if let Some(script) = self.init_script.take() {
            log::info!("Initializing Game Executor...");
            self.driver = Some(ExecutorHandle::new(&mut self.ctx, script));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(_) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.invalidate_swapchain();
                    renderer.window.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.prepare_swapchain();

                    // --- 游戏逻辑步进 ---
                    if let Some(driver) = self.driver.as_mut() {
                        let _waiting = driver.step(&mut self.ctx);

                        // 处理非视觉事件 (音频等)
                        for event in self.ctx.drain() {
                            match event {
                                OutputEvent::End => event_loop.exit(),
                                OutputEvent::PlayAudio { .. } => {
                                    // TODO: 对接音频系统
                                    log::info!("(Audio) {:?}", event);
                                }
                                _ => {}
                            }
                        }
                    }

                    // --- 核心绘制 ---
                    // 必须先解构借用，避免在闭包中同时借用 &mut self.renderer 和 &mut self.painter
                    let painter = &mut self.painter;
                    let ctx = &self.ctx;

                    renderer.draw_and_present(|canvas, size| {
                        // 将 winit 的 size (LogicalSize) 转换为 painter 需要的 (width, height)
                        painter.paint(canvas, ctx, (size.width, size.height));
                    });

                }
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Space),
                    ..
                },
                ..
            } => {
                if let Some(driver) = self.driver.as_mut() {
                    driver.feed(&mut self.ctx, InputEvent::Continue);
                    // 触发重绘
                    if let Some(renderer) = self.renderer.as_ref() {
                        renderer.window.request_redraw();
                    }
                }
            }

            _ => {
                // 持续刷新以播放动画或响应调整
                if let Some(renderer) = self.renderer.as_ref() {
                    renderer.window.request_redraw();
                }
            }
        }
    }
}