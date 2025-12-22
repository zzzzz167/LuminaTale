use crate::core::{Painter, AssetManager, AudioPlayer};
use crate::vk_utils::context::VulkanRenderContext;
use crate::vk_utils::renderer::VulkanRenderer;
use crate::scene::{AppScene, SceneAnimator};
use crate::config::WindowConfig;
use crate::ui::UiDrawer;

use winit::{
    event_loop::{ControlFlow, EventLoop, ActiveEventLoop},
    application::ApplicationHandler,
    window::{Window, WindowId},
    event::{WindowEvent, ElementState, MouseButton},
    dpi::PhysicalSize
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use log::debug;
use lumina_shared;
use viviscript_core::ast::Script;
use lumina_core::{Ctx, OutputEvent};
use lumina_core::event::InputEvent;
use lumina_core::renderer::driver::ExecutorHandle;
use lumina_ui::{Rect, input::UiContext};

// 设计分辨率
const DESIGN_WIDTH: f32 = 1920.0;
const DESIGN_HEIGHT: f32 = 1080.0;

pub struct SkiaRenderer {
    render_ctx: VulkanRenderContext,
    renderer: Option<VulkanRenderer>,
    assets: AssetManager,
    audio_player: AudioPlayer,
    painter: Painter,
    animator: SceneAnimator,

    game_script: Arc<Script>,
    state: AppScene,

    ui_ctx: UiContext,

    physical_cursor_pos: (f32, f32),
    scale_factor: f64,

    active_choices: Option<(Option<String>, Vec<String>)>,

    gc_timer: Instant,
    last_frame: Instant,
}

impl SkiaRenderer {
    pub fn new(script: Arc<Script>) -> Self {
        let cfg: WindowConfig = lumina_shared::config::get("window");
        let asset_path = &cfg.assets.assets_path;

        let initial_state = if cfg.debug.skip_main_menu {
            let mut ctx = Ctx::default();
            let driver = ExecutorHandle::new(&mut ctx, script.clone());
            AppScene::InGame { ctx, driver, }
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

            game_script: script,
            state: initial_state,

            ui_ctx: UiContext::new(),
            physical_cursor_pos: (0.0, 0.0),
            scale_factor: 1.0,
            active_choices: None,

            gc_timer: Instant::now(),
            last_frame: Instant::now(),
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


    fn to_logical(physical_x: f32, physical_y: f32, scale: f32, off_x: f32, off_y: f32) -> (f32, f32) {
        if scale == 0.0 { return (0.0, 0.0); }
        (
            (physical_x - off_x) / scale,
            (physical_y - off_y) / scale
        )
    }

    fn update_ingame(&mut self, dt: f32, event_loop: &ActiveEventLoop) {
        if let AppScene::InGame {ctx, driver} = &mut self.state {
            let mut waiting = false;
            for _ in 0..100 {
                waiting = driver.step(ctx);
                if waiting { break; }
            }

            // Make the compiler happy.
            let events: Vec<_> = ctx.drain().into_iter().collect();

            let get_sprite_info = |target: &str| -> (Option<String>, Option<Vec<String>>) {
                if let Some(layer) = ctx.layer_record.layer.get("master") {
                    if let Some(s) = layer.iter().find(|s| s.target == target) {
                        return (s.position.clone(), Some(s.attrs.clone()));
                    }
                }
                (None, None)
            };

            for event in events {
                match event {
                    OutputEvent::PlayAudio { channel, path, fade_in, volume, looping } => {
                        if let Some(full_path) = self.assets.get_audio_path(&path) {
                            self.audio_player.play(&channel, full_path, volume, fade_in, looping);
                        }
                    },
                    OutputEvent::StopAudio { channel, fade_out } => {
                        self.audio_player.stop(&channel, fade_out);
                    },
                    OutputEvent::NewSprite { target, transition } => {
                        let texture_name = ctx.characters.get(&target)
                            .and_then(|ch| ch.image_tag.clone())
                            .unwrap_or_else(|| target.clone());
                        let (pos_str, attrs) = get_sprite_info(&target);
                        // 如果 attrs 是 None，给个空 Vec
                        let attrs = attrs.unwrap_or_default();

                        self.animator.handle_new_sprite(target, texture_name, transition, pos_str.as_deref(), attrs);
                    },
                    OutputEvent::UpdateSprite { target, transition } => {
                        // ✅ 修复：复用 helper 获取最新位置
                        let (pos_str, attrs) = get_sprite_info(&target);
                        self.animator.handle_update_sprite(target, transition, pos_str.as_deref(), attrs);
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
                        self.active_choices = Some((title, options));
                    },
                    OutputEvent::ShowDialogue { .. } | OutputEvent::ShowNarration { .. } => {
                        self.active_choices = None;
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
        let window_attributes = Window::default_attributes()
            .with_title(&cfg.title)
            .with_inner_size(PhysicalSize::new(cfg.width, cfg.height))
            .with_resizable(cfg.resizable);

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.scale_factor = window.scale_factor();

        self.animator.resize(DESIGN_WIDTH, DESIGN_HEIGHT);
        self.renderer = Some(self.render_ctx.renderer_for_window(event_loop, window.clone(), cfg.vsync));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(_) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.invalidate_swapchain();
                    self.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                self.request_redraw();
            },
            // 1. 鼠标移动：记录物理坐标
            WindowEvent::CursorMoved { position, .. } => {
                // 注意：这里必须是 f32
                self.physical_cursor_pos = (position.x as f32, position.y as f32);
                self.request_redraw();
            },

            // 2. 点击：记录状态
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                let pressed = state == ElementState::Pressed;
                self.ui_ctx.mouse_pressed = pressed && !self.ui_ctx.mouse_held;
                self.ui_ctx.mouse_held = pressed;
                self.request_redraw();
            },

            WindowEvent::RedrawRequested => {
                // 先更新逻辑，避免借用冲突
                let now = Instant::now();
                let dt = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;
                self.update_ingame(dt, event_loop);

                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.prepare_swapchain();

                    let dpi = self.scale_factor as f32;

                    let ui_ctx_ref = &mut self.ui_ctx;
                    let state_ref = &mut self.state;
                    let game_script_ref = &self.game_script;
                    let active_choices_ref = &mut self.active_choices;
                    let painter = &mut self.painter;
                    let assets = &mut self.assets;
                    let animator = &self.animator;

                    // 获取刚才记录的物理坐标
                    let (mx, my) = self.physical_cursor_pos;

                    renderer.draw_and_present(|canvas, size| {
                        // A. 计算并应用黑边适配
                        let win_w = size.width as f32;
                        let win_h = size.height as f32;

                        // B. 计算逻辑坐标
                        let (adj_mx, adj_my) = if dpi > 1.0 {
                            (mx / dpi, my / dpi)
                        } else {
                            (mx, my)
                        };

                        // 计算布局缩放
                        let scale_x = win_w / DESIGN_WIDTH;
                        let scale_y = win_h / DESIGN_HEIGHT;
                        let scale = scale_x.min(scale_y);

                        let off_x = (win_w - DESIGN_WIDTH * scale) / 2.0;
                        let off_y = (win_h - DESIGN_HEIGHT * scale) / 2.0;

                        // C. 计算最终逻辑坐标 (传入调整后的鼠标坐标)
                        let (lx, ly) = SkiaRenderer::to_logical(adj_mx, adj_my, scale, off_x, off_y);

                        // 更新 UI 上下文
                        ui_ctx_ref.update(lx, ly, ui_ctx_ref.mouse_pressed, ui_ctx_ref.mouse_held);

                        // C. Skia 变换 (视觉层)
                        canvas.save();
                        canvas.translate(skia_safe::Vector::new(off_x, off_y));
                        canvas.scale((scale, scale));

                        // 裁剪显示区域
                        canvas.clip_rect(skia_safe::Rect::new(0.0, 0.0, DESIGN_WIDTH, DESIGN_HEIGHT), None, None);

                        // D. 绘制逻辑 (坐标系 1920x1080)
                        match state_ref {
                            AppScene::MainMenu => {
                                canvas.clear(skia_safe::Color::BLACK);
                                let screen = Rect::new(0.0, 0.0, DESIGN_WIDTH, DESIGN_HEIGHT);
                                let mut ui = UiDrawer::new(canvas, ui_ctx_ref, &painter.font_collection);

                                let menu_area = screen.center(400.0, 500.0);
                                let (title_rect, content) = menu_area.split_top(150.0);
                                ui.label("Lumina Tale", title_rect, 60.0, skia_safe::Color::WHITE);

                                let (btn1, rest) = content.split_top(80.0);
                                let (btn2, rest) = rest.split_top(80.0);
                                let (btn3, _)    = rest.split_top(80.0);

                                if ui.button("Start Game", btn1.shrink(10.0)) {
                                    log::info!("Starting Game...");
                                    let mut ctx = Ctx::default();
                                    let driver = ExecutorHandle::new(&mut ctx, game_script_ref.clone());
                                    *state_ref = AppScene::InGame { ctx, driver };
                                    *active_choices_ref = None;
                                }
                                if ui.button("Settings", btn2.shrink(10.0)) {
                                    // Settings
                                }
                                if ui.button("Quit", btn3.shrink(10.0)) {
                                    std::process::exit(0);
                                }
                            },
                            AppScene::InGame { ctx, driver } => {
                                let screen = Rect::new(0.0, 0.0, DESIGN_WIDTH, DESIGN_HEIGHT);

                                painter.paint(canvas, ctx, animator, (DESIGN_WIDTH, DESIGN_HEIGHT), assets);

                                let mut ui = UiDrawer::new(canvas, ui_ctx_ref, &painter.font_collection);

                                // 临时变量存储点击结果
                                let mut choice_made_index = None;

                                if let Some((title, options)) = active_choices_ref {
                                    let mut p = skia_safe::Paint::default();
                                    p.set_color(skia_safe::Color::from_argb(200, 0, 0, 0));
                                    canvas.draw_rect(skia_safe::Rect::new(0.0, 0.0, DESIGN_WIDTH, DESIGN_HEIGHT), &p);

                                    let menu = screen.center(600.0, 600.0);
                                    let (header, mut body) = menu.split_top(100.0);
                                    if let Some(t) = title { ui.label(t, header, 40.0, skia_safe::Color::WHITE); }

                                    for (idx, txt) in options.iter().enumerate() {
                                        let (btn, rest) = body.split_top(80.0);
                                        body = rest;
                                        if ui.button(txt, btn.shrink(10.0)) {
                                            choice_made_index = Some(idx);
                                        }
                                    }
                                } else {
                                    if ui_ctx_ref.mouse_pressed {
                                        driver.feed(ctx, InputEvent::Continue);
                                    }
                                }

                                // 延迟处理点击，避免借用冲突
                                if let Some(idx) = choice_made_index {
                                    driver.feed(ctx, InputEvent::ChoiceMade{index: idx});
                                    *active_choices_ref = None;
                                }
                            },
                            _ => {}
                        }

                        canvas.restore();
                    });

                    self.ui_ctx.mouse_pressed = false;

                    if self.gc_timer.elapsed().as_secs() >= 5 {
                        self.assets.gc(Duration::from_secs(10));
                        self.gc_timer = Instant::now();
                    }
                    renderer.window.request_redraw();
                }
            },
            _ => {}
        }
    }
}