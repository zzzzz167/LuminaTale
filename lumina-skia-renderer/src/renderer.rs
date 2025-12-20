use std::sync::Arc;
use crate::core::{Painter, AssetManager, AudioPlayer};
use crate::vk_utils::context::VulkanRenderContext;
use crate::vk_utils::renderer::VulkanRenderer;
use crate::ui_state::{UiMode, UiState};
use crate::scene::{AppScene, SceneAnimator};
use crate::config::WindowConfig;

use winit::{
    event_loop::{ControlFlow, EventLoop, ActiveEventLoop},
    application::ApplicationHandler,
    window::{Window, WindowId},
    event::{WindowEvent, KeyEvent, ElementState, MouseButton},
    keyboard::{PhysicalKey, KeyCode},
    dpi::PhysicalSize
};
use std::time::{Duration, Instant};
use skia_safe::{Contains, Point};

use lumina_shared;
use viviscript_core::ast::Script;
use lumina_core::{Ctx, OutputEvent};
use lumina_core::event::InputEvent;
use lumina_core::renderer::driver::ExecutorHandle;

pub struct SkiaRenderer {
    render_ctx: VulkanRenderContext,
    renderer: Option<VulkanRenderer>,
    assets: AssetManager,
    audio_player: AudioPlayer,
    painter: Painter,
    animator: SceneAnimator,

    state: AppScene,

    gc_timer: Instant,
    last_frame: Instant,
    cursor_pos: Point,
}

impl SkiaRenderer {
    pub fn new(script: Option<Script>) -> Self {
        let cfg: WindowConfig = lumina_shared::config::get("window");
        let asset_path = &cfg.assets.assets_path;

        let initial_state = if let Some(s) = script {
            let mut ctx = Ctx::default();
            let driver = ExecutorHandle::new(&mut ctx, s);
            AppScene::InGame {
                ctx,
                driver,
                ui_state: UiState::default()
            }
        } else {
                AppScene::MainMenu
        };

        Self {
            render_ctx: VulkanRenderContext::default(),
            renderer: None,
            assets: AssetManager::new(asset_path),
            audio_player: AudioPlayer::new(),
            painter: Painter::new(),
            animator: SceneAnimator::new(),

            state: initial_state,

            gc_timer: Instant::now(),
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

    fn update_ingame(&mut self, dt: f32, event_loop: &ActiveEventLoop) {
        if let AppScene::InGame {ctx, driver, ui_state} = &mut self.state {
            let mut waiting = false;
            for _ in 0..1000 {
                waiting = driver.step(ctx);
                if waiting { break; } // 遇到 WaitInput 或 WaitChoice，停止执行，等待渲染和输入
            }

            for event in ctx.drain() {
                match event {
                    OutputEvent::PlayAudio { channel, path, fade_in, volume, looping } => {
                        if let Some(full_path) = self.assets.get_audio_path(&path) {
                            self.audio_player.play(&channel, full_path, volume, fade_in, looping);
                        } else {
                            log::error!("Audio not found: {}", path);
                        }
                    },
                    OutputEvent::StopAudio { channel, fade_out } => {
                        self.audio_player.stop(&channel, fade_out);
                    },
                    OutputEvent::NewSprite { target, transition } => {
                        let texture_name = ctx.characters.get(&target)
                            .and_then(|ch| ch.image_tag.clone())
                            .unwrap_or_else(|| target.clone());
                        let mut pos_str = None;
                        let mut attrs = Vec::new();
                        if let Some(layer) = ctx.layer_record.layer.get("master") {
                            if let Some(sprite) = layer.iter().find(|s| s.target == target) {
                                pos_str = sprite.position.as_deref();
                                attrs = sprite.attrs.clone();
                            }
                        }
                        self.animator.handle_new_sprite(target, texture_name, transition, pos_str, attrs);
                    },
                    OutputEvent::UpdateSprite { target, transition } => {
                        let mut pos_str = None;
                        let mut new_attrs = None;

                        if let Some(layer) = ctx.layer_record.layer.get("master") {
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
                        if let Some(layer) = ctx.layer_record.layer.get("master") {
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
                        log::info!("[UI] Entering Choice Mode");
                        ui_state.set_choices(title, options);
                    },
                    OutputEvent::ShowDialogue { .. } | OutputEvent::ShowNarration { .. } => {
                        if ui_state.is_choosing() { ui_state.clear(); }
                    },
                    OutputEvent::End => event_loop.exit(),
                    _ => {}
                }
            }
        }

        self.animator.update(dt);
    }
}

impl ApplicationHandler for SkiaRenderer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let cfg: WindowConfig = lumina_shared::config::get("window");
        log::info!("Window Config Loaded: {}x{} VSync:{}", cfg.width, cfg.height, cfg.vsync);

        let window_attributes = Window::default_attributes()
            .with_title(&cfg.title)
            .with_inner_size(PhysicalSize::new(cfg.width, cfg.height))
            .with_resizable(cfg.resizable);

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let size = window.inner_size();
        let scale_factor = window.scale_factor();
        let logical_size = size.to_logical::<f32>(scale_factor);

        log::debug!("Window Init: Physical {:?}, Logical {:?}", size, logical_size);
        self.animator.resize(logical_size.width, logical_size.height);

        self.renderer = Some(self.render_ctx.renderer_for_window(event_loop, window.clone(), cfg.vsync));
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
                }

                let now = Instant::now();
                let dt = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;

                match &mut self.state {
                    AppScene::MainMenu => {self.animator.update(dt);},
                    AppScene::InGame {..} => {
                        self.update_ingame(dt, event_loop);
                    }
                };

                if let Some(renderer) = self.renderer.as_mut() {

                    // --- 核心绘制 ---
                    // 必须先解构借用，避免在闭包中同时借用 &mut self.renderer 和 &mut self.painter
                    let painter = &mut self.painter;
                    let assets = &mut self.assets;
                    let animator = &self.animator;
                    let state = &mut self.state;

                    renderer.draw_and_present(|canvas, size| {
                        match state {
                            AppScene::MainMenu => {
                                canvas.clear(skia_safe::Color::from_argb(255, 40, 40, 40));
                            },
                            AppScene::InGame { ctx, ui_state, .. } => {
                                painter.paint(canvas, ctx, animator, ui_state, (size.width, size.height), assets);
                            }
                        }
                    });

                    if self.gc_timer.elapsed().as_secs() >= 5 {
                        self.assets.gc(Duration::from_secs(10));
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

                match &mut self.state {
                    AppScene::MainMenu => {
                        // TODO: 处理主菜单按钮 Hover
                    },
                    AppScene::InGame { ui_state, .. } => {
                        // 现有的 Choice Hover 逻辑
                        if let UiMode::Choice { hit_boxes, hover_index, .. } = &mut ui_state.mode {
                            let mut new_hover = None;
                            for (i, rect) in hit_boxes.iter().enumerate() {
                                if rect.contains(self.cursor_pos) { new_hover = Some(i); break; }
                            }
                            if *hover_index != new_hover { *hover_index = new_hover; self.request_redraw(); }
                        }
                    }
                }
            },

            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                match &mut self.state {
                    AppScene::MainMenu => {
                        log::info!("Clicked on Main Menu!");
                    },
                    AppScene::InGame { ctx, driver, ui_state } => {
                        // 现有的游戏点击逻辑
                        match &ui_state.mode {
                            UiMode::Choice { hover_index: Some(idx), .. } => {
                                driver.feed(ctx, InputEvent::ChoiceMade { index: *idx });
                                ui_state.clear();
                                self.request_redraw();
                            },
                            UiMode::None => {
                                driver.feed(ctx, InputEvent::Continue);
                                self.request_redraw();
                            }
                            _ => {}
                        }
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
                match &mut self.state {
                    AppScene::MainMenu => {
                        // TODO
                    },
                    AppScene::InGame { ctx, driver ,ui_state, .. } => {
                        if !ui_state.is_choosing() {
                            driver.feed(ctx, InputEvent::Continue);
                            self.request_redraw();
                        }
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