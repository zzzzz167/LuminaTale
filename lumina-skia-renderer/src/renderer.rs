use crate::core::{Painter, AssetManager, AudioPlayer};
use crate::vk_utils::context::VulkanRenderContext;
use crate::vk_utils::renderer::VulkanRenderer;
use crate::ui_state::{UiState, UiMode};
use crate::scene::{AppScene, SceneAnimator};
use crate::config::WindowConfig;
use crate::ui::{UiAction, WidgetNode};

use winit::{
    event_loop::{ControlFlow, EventLoop, ActiveEventLoop},
    application::ApplicationHandler,
    window::{Window, WindowId},
    event::{WindowEvent, KeyEvent, ElementState, MouseButton},
    keyboard::{PhysicalKey, KeyCode},
    dpi::PhysicalSize
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use skia_safe::{Point, Rect};

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

    game_script: Arc<Script>,
    state: AppScene,

    gc_timer: Instant,
    last_frame: Instant,
    cursor_pos: Point,
}

impl SkiaRenderer {
    pub fn new(script: Arc<Script>) -> Self {
        let cfg: WindowConfig = lumina_shared::config::get("window");
        let asset_path = &cfg.assets.assets_path;

        let initial_state = if cfg.debug.skip_main_menu {
            let mut ctx = Ctx::default();
            let driver = ExecutorHandle::new(&mut ctx, script.clone());
            AppScene::InGame {
                ctx,
                driver,
                ui_state: UiState::default()
            }
        } else {
            AppScene::default()
        };

        Self {
            render_ctx: VulkanRenderContext::default(),
            renderer: None,
            assets: AssetManager::new(asset_path),
            audio_player: AudioPlayer::new(),
            painter: Painter::new(),
            animator: SceneAnimator::new(),

            game_script: script,
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

    fn handle_ui_action(&mut self, action: UiAction, event_loop: &ActiveEventLoop) {
        match action {
            // FIXME: 这块的架构太奇怪了, 应该重新设计
            UiAction::Quit => event_loop.exit(),

            UiAction::OpenMenu(name) => {
                if name == "Settings" {
                    let prev = std::mem::replace(&mut self.state, AppScene::default());
                    self.state = AppScene::new_settings(prev);
                }
            },

            UiAction::Back => {
                let prev_state = if let AppScene::Settings { prev_scene, .. } = &mut self.state {
                    Some(std::mem::replace(prev_scene.as_mut(), AppScene::default()))
                } else {
                    None
                };

                if let Some(prev) = prev_state {
                    self.state = prev;
                }
            },

            UiAction::RunScript(cmd) => {
                if cmd == "StartGame" {
                    log::info!("Action: Start Game");
                    let mut ctx = Ctx::default();
                    let driver = ExecutorHandle::new(&mut ctx, self.game_script.clone());

                    self.state = AppScene::InGame {
                        ctx,
                        driver,
                        ui_state: UiState::default()
                    };
                }
            },

            UiAction::ScriptChoice(idx) => {
                if let AppScene::InGame { ctx, driver, ui_state } = &mut self.state {
                    driver.feed(ctx, InputEvent::ChoiceMade { index: idx });
                    ui_state.clear();
                }
            },
            UiAction::AdjustVolume(key, val) => {
                log::debug!("Volume [{}] -> {:.2}", key, val);
                // TODO: 接入 AudioPlayer 和 Config
                // FIXME: 这块的处理很不好其实
                // self.audio_player.set_volume(key, val);
            },

            UiAction::ToggleConfig(key) => {
                log::debug!("Toggle Config [{}]", key);
                // TODO: 接入 Config
            },
            _ => {}
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
                    AppScene::InGame { .. } => self.update_ingame(dt, event_loop),
                    _=> {self.animator.update(dt);}
                };

                if let Some(renderer) = self.renderer.as_mut() {

                    // --- 核心绘制 ---
                    // 必须先解构借用，避免在闭包中同时借用 &mut self.renderer 和 &mut self.painter
                    let painter = &mut self.painter;
                    let assets = &mut self.assets;
                    let animator = &self.animator;
                    let state = &mut self.state;

                    renderer.draw_and_present(|canvas, size| {
                        let mut dummy_ctx = Ctx::default();
                        let mut dummy_ui_state = UiState::default();

                        let win_rect = Rect::from_wh(size.width as f32, size.height as f32);

                        let (ctx, ui_state, ui_root) = match state {
                            AppScene::InGame { ctx, ui_state, .. } => {
                                (ctx, ui_state, None)
                            },
                            AppScene::MainMenu { root, .. } => {
                                root.layout(win_rect);
                                (&mut dummy_ctx, &mut dummy_ui_state, Some(root as &WidgetNode))
                            },
                            AppScene::Settings { root, .. } => {
                                root.layout(win_rect);
                                (&mut dummy_ctx, &mut dummy_ui_state, Some(root as &WidgetNode))
                            },
                        };

                        painter.paint(
                            canvas,
                            ctx,
                            animator,
                            ui_state,
                            (size.width, size.height),
                            assets,
                            ui_root
                        );
                    });

                    if self.gc_timer.elapsed().as_secs() >= 5 {
                        self.assets.gc(Duration::from_secs(10));
                        self.gc_timer = Instant::now();
                    }
                    renderer.window.request_redraw();
                }
            },

            WindowEvent::CursorMoved { position, .. } => {
                let scale = self.renderer.as_ref().map(|r| r.window.scale_factor()).unwrap_or(1.0);
                self.cursor_pos = Point::new(position.x as f32 / scale as f32, position.y as f32 / scale as f32);

                let mut action_to_perform = UiAction::None;

                match &mut self.state {
                    AppScene::MainMenu { root } => {
                        action_to_perform = root.on_mouse_move(self.cursor_pos);
                    },
                    AppScene::Settings { root, .. } => {
                        action_to_perform = root.on_mouse_move(self.cursor_pos);
                    },
                    AppScene::InGame { ui_state, .. } => {
                        if let UiMode::Choice { root, .. } = &mut ui_state.mode {
                            action_to_perform = root.on_mouse_move(self.cursor_pos);
                        }
                    }
                }
                if action_to_perform != UiAction::None {
                    self.handle_ui_action(action_to_perform, event_loop);
                }
                self.request_redraw();
            },

            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let mut action_to_perform = UiAction::None;

                match &mut self.state {
                    AppScene::MainMenu { root } => {
                        action_to_perform = root.on_click(self.cursor_pos);
                    },
                    AppScene::Settings { root, .. } => {
                        action_to_perform = root.on_click(self.cursor_pos);
                    },
                    AppScene::InGame { ctx, driver, ui_state } => {
                        // 1. 先看有没有点到 UI (选项)
                        if let UiMode::Choice { root, .. } = &mut ui_state.mode {
                            action_to_perform = root.on_click(self.cursor_pos);
                        }

                        // 2. 没点到 UI，就是普通交互 (继续对话)
                        if action_to_perform == UiAction::None && !ui_state.is_choosing() {
                            driver.feed(ctx, InputEvent::Continue);
                            self.request_redraw();
                        }
                    }
                }

                if action_to_perform != UiAction::None {
                    self.handle_ui_action(action_to_perform, event_loop);
                    self.request_redraw();
                }
            },
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                match &mut self.state {
                    AppScene::MainMenu { root } => root.on_mouse_release(),
                    AppScene::Settings { root, .. } => root.on_mouse_release(),
                    AppScene::InGame { ui_state, .. } => {
                        if let UiMode::Choice { root, .. } = &mut ui_state.mode {
                            root.on_mouse_release();
                        }
                    }
                }
            },

            // TODO: 完善此处对键盘输入的处理
            /*
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Space),
                    ..
                },
                ..
            } => {
                match &mut self.state {
                    AppScene::MainMenu { .. } => {
                    },
                    AppScene::InGame { ctx, driver ,ui_state, .. } => {
                        if !ui_state.is_choosing() {
                            driver.feed(ctx, InputEvent::Continue);
                            self.request_redraw();
                        }
                    }
                }
            }
            */

            _ => {
                // 持续刷新以播放动画或响应调整
                if let Some(renderer) = self.renderer.as_ref() {
                    renderer.window.request_redraw();
                }
            }
        }
    }
}