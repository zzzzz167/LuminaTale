use crate::painter::Painter;
use crate::vk_utils::context::VulkanRenderContext;
use crate::vk_utils::renderer::VulkanRenderer;
use crate::ui_state::{UiMode, UiState};
use crate::animator::SceneAnimator;

use winit::{
    event_loop::{ControlFlow, EventLoop, ActiveEventLoop},
    application::ApplicationHandler,
    window::{Window, WindowId},
    event::{WindowEvent, KeyEvent, ElementState, MouseButton},
    keyboard::{PhysicalKey, KeyCode}
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use skia_safe::{Contains, Point};

use viviscript_core::ast::Script;
use lumina_core::{Ctx, OutputEvent};
use lumina_core::event::InputEvent;
use lumina_core::renderer::driver::ExecutorHandle;

pub struct SkiaRenderer {
    render_ctx: VulkanRenderContext,
    renderer: Option<VulkanRenderer>,
    painter: Painter,
    ui_state: UiState,
    gc_timer: Instant,

    ctx: Ctx,
    driver: Option<ExecutorHandle>,
    init_script: Option<Script>,

    animator: SceneAnimator,
    last_frame: Instant,

    cursor_pos: Point,
}

impl SkiaRenderer {
    pub fn new(script: Script) -> Self {
        Self {
            render_ctx: VulkanRenderContext::default(),
            renderer: None,
            painter: Painter::new(),
            ui_state: UiState::default(),
            gc_timer: Instant::now(),
            ctx: Ctx::default(),
            driver: None,
            init_script: Some(script),
            animator: SceneAnimator::new(),
            last_frame: Instant::now(),
            cursor_pos: Point::new(0.0, 0.0),
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).unwrap();
    }

    fn request_redraw(&self) {
        if let Some(renderer) = self.renderer.as_ref() {
            renderer.window.request_redraw();
        }
    }
}

impl ApplicationHandler for SkiaRenderer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes().with_title("LuminaTale Skia")).unwrap());

        let size = window.inner_size();
        let scale_factor = window.scale_factor();
        let logical_size = size.to_logical::<f32>(scale_factor);

        log::debug!("Window Init: Physical {:?}, Logical {:?}", size, logical_size);
        self.animator.resize(logical_size.width, logical_size.height);

        self.renderer = Some(self.render_ctx.renderer_for_window(event_loop, window.clone()));
        if let Some(script) = self.init_script.take() {
            log::info!("Initializing Game Executor...");
            self.driver = Some(ExecutorHandle::new(&mut self.ctx, script));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.invalidate_swapchain();

                    let scale_factor = renderer.window.scale_factor();
                    let logical_size = size.to_logical::<f32>(scale_factor);

                    self.animator.resize(logical_size.width, logical_size.height);
                    self.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.prepare_swapchain();

                    let now = Instant::now();
                    let dt = now.duration_since(self.last_frame).as_secs_f32();
                    self.last_frame = now;

                    // --- 游戏逻辑步进 ---
                    if let Some(driver) = self.driver.as_mut() {
                        let mut waiting = false;
                        for _ in 0..1000 {
                            waiting = driver.step(&mut self.ctx);
                            if waiting { break; } // 遇到 WaitInput 或 WaitChoice，停止执行，等待渲染和输入
                        }

                        // 处理非视觉事件 (音频等)
                        for event in self.ctx.drain() {
                            match event {
                                OutputEvent::NewSprite { target, transition } => {
                                    let texture_name = self.ctx.characters.get(&target)
                                        .and_then(|ch| ch.image_tag.clone())
                                        .unwrap_or_else(|| target.clone());

                                    let mut pos_str = None;
                                    let mut attrs = Vec::new();
                                    if let Some(layer) = self.ctx.layer_record.layer.get("master") {
                                        if let Some(sprite) = layer.iter().find(|s| s.target == target) {
                                            pos_str = sprite.position.as_deref();
                                        }
                                    }

                                    if let Some(layer) = self.ctx.layer_record.layer.get("master") {
                                        if let Some(sprite) = layer.iter().find(|s| s.target == target) {
                                            pos_str = sprite.position.as_deref();
                                            attrs = sprite.attrs.clone(); // [新增] 提取属性
                                        }
                                    }

                                    self.animator.handle_new_sprite(target, texture_name, transition, pos_str, attrs);
                                },
                                OutputEvent::UpdateSprite { target, transition } => {
                                    let mut pos_str = None;
                                    let mut new_attrs = None;

                                    if let Some(layer) = self.ctx.layer_record.layer.get("master") {
                                        if let Some(sprite) = layer.iter().find(|s| s.target == target) {
                                            pos_str = sprite.position.as_deref();
                                            new_attrs = Some(sprite.attrs.clone());
                                        }
                                    }

                                    self.animator.handle_update_sprite(target, transition, pos_str, new_attrs);
                                },
                                OutputEvent::HideSprite { target, transition } => {
                                    self.animator.handle_hide_sprite(target, transition);
                                },

                                OutputEvent::NewScene { transition } => {
                                    let mut bg_name = None;

                                    if let Some(layer) = self.ctx.layer_record.layer.get("master") {
                                        if let Some(bg) = layer.first() {
                                            let mut full_name = bg.target.clone();
                                            if !bg.attrs.is_empty() {
                                                full_name.push('_');
                                                full_name.push_str(&bg.attrs.join("_"));
                                            }
                                            bg_name = Some(full_name);
                                        }
                                    }

                                    self.animator.handle_new_scene(bg_name, transition);
                                },

                                OutputEvent::ShowChoice { title, options } => {
                                    log::info!("UI: Entering Choice Mode");
                                    self.ui_state.set_choices(title, options);
                                },
                                OutputEvent::ShowDialogue { .. } | OutputEvent::ShowNarration { .. } => {
                                    // 任何新文本出现，清理旧的 Choice 状态（安全起见）
                                    if self.ui_state.is_choosing() {
                                        self.ui_state.clear();
                                    }
                                },
                                OutputEvent::End => event_loop.exit(),
                                OutputEvent::PlayAudio { .. } => {
                                    // TODO: 对接音频系统
                                    log::info!("(Audio) {:?}", event);
                                }
                                _ => {}
                            }
                        }
                    }
                    self.animator.update(dt);

                    // --- 核心绘制 ---
                    // 必须先解构借用，避免在闭包中同时借用 &mut self.renderer 和 &mut self.painter
                    let painter = &mut self.painter;
                    let animator = &self.animator; // 传 animator
                    let ui = &mut self.ui_state;
                    let ctx = &self.ctx;

                    renderer.draw_and_present(|canvas, size| {
                        painter.paint(canvas, ctx, animator, ui, (size.width, size.height));
                    });

                    if self.gc_timer.elapsed().as_secs() >= 5 {
                        self.painter.gc_assets(Duration::from_secs(10));
                        self.gc_timer = Instant::now();
                    }

                    self.request_redraw();
                }
            },

            WindowEvent::CursorMoved { position, .. } => {
                let scale_factor = self.renderer.as_ref()
                    .map(|r| r.window.scale_factor())
                    .unwrap_or(1.0);

                let x = position.x as f32 / scale_factor as f32;
                let y = position.y as f32 / scale_factor as f32;
                self.cursor_pos = Point::new(x, y);

                if let UiMode::Choice { hit_boxes, hover_index, .. } = &mut self.ui_state.mode {
                    let mut new_hover = None;
                    for (i, rect) in hit_boxes.iter().enumerate() {
                        // 检测点是否在矩形内
                        if rect.contains(self.cursor_pos) {
                            new_hover = Some(i);
                            break;
                        }
                    }

                    // 状态改变才重绘，节省性能
                    if *hover_index != new_hover {
                        *hover_index = new_hover;
                        self.request_redraw();
                    }
                }
            },

            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                if let Some(driver) = self.driver.as_mut() {
                    match &self.ui_state.mode {
                        UiMode::Choice { hover_index:Some(idx), .. } => {
                            log::debug!("UI: Choice made -> {}", idx);
                            driver.feed(&mut self.ctx, InputEvent::ChoiceMade { index: *idx });

                            self.ui_state.clear();
                            self.request_redraw();
                        },
                        UiMode::None => {
                            driver.feed(&mut self.ctx, InputEvent::Continue);
                            self.request_redraw();
                        }
                        _ => {}
                    }
                }
            },

            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Space),
                    ..
                },
                ..
            } => {
                if let Some(driver) = self.driver.as_mut() {
                    if !self.ui_state.is_choosing() {
                        driver.feed(&mut self.ctx, InputEvent::Continue);
                        self.request_redraw();
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