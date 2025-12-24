use super::{Screen, ScreenTransition};
use crate::ui::UiDrawer;
use crate::core::{AssetManager, Painter, AudioPlayer};
use crate::core::SceneAnimator;
use lumina_core::{Ctx, OutputEvent};
use lumina_core::event::InputEvent;
use lumina_core::renderer::driver::ExecutorHandle;
use lumina_ui::{Rect, Color, UiRenderer, Alignment, GradientDirection};
use lumina_ui::widgets::{Button, Label, Panel};
use winit::event_loop::ActiveEventLoop;

pub struct InGameScreen {
    driver: ExecutorHandle,
    animator: SceneAnimator,
    active_choices: Option<(Option<String>, Vec<String>)>,
}

impl InGameScreen {
    pub fn new(driver: ExecutorHandle) -> Self {
        let mut animator = SceneAnimator::new();
        animator.resize(1920.0, 1080.0);

        Self {
            driver,
            animator,
            active_choices: None,
        }
    }

    /// 处理 Core 输出的事件 (Audio, Sprite, Dialogue...)
    fn process_output_events(
        &mut self,
        ctx: &mut Ctx,
        el: &ActiveEventLoop,
        assets: &AssetManager,
        audio: &mut AudioPlayer
    ) {
        // 1. 收集事件，解开 ctx 的借用锁
        let events: Vec<_> = ctx.drain().into_iter().collect();

        // 辅助闭包：获取 Sprite 初始状态
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
                // --- 音频处理 ---
                OutputEvent::PlayAudio { channel, path, fade_in, volume, looping } => {
                    if let Some(full_path) = assets.get_audio_path(&path) {
                        audio.play(&channel, full_path, volume, fade_in, looping);
                    }
                },
                OutputEvent::StopAudio { channel, fade_out } => {
                    audio.stop(&channel, fade_out);
                },

                // --- 视觉处理 (委托给 Animator) ---
                OutputEvent::NewSprite { target, transition } => {
                    let texture_name = ctx.characters.get(&target)
                        .and_then(|ch| ch.image_tag.clone())
                        .unwrap_or_else(|| target.clone());

                    let (pos_str, attrs) = get_sprite_info(&target);
                    let attrs = attrs.unwrap_or_default();

                    self.animator.handle_new_sprite(
                        target,
                        texture_name,
                        transition,
                        pos_str.as_deref(),
                        attrs
                    );
                },
                OutputEvent::UpdateSprite { target, transition } => {
                    let (pos_str, attrs) = get_sprite_info(&target);
                    let attrs = attrs.unwrap_or_default();

                    self.animator.handle_update_sprite(
                        target,
                        transition,
                        pos_str.as_deref(),
                        attrs
                    );
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

                // --- 流程控制 ---
                OutputEvent::ShowChoice { title, options } => {
                    self.active_choices = Some((title, options));
                },
                OutputEvent::ShowDialogue { .. } | OutputEvent::ShowNarration { .. } => {
                    // 进入对话时，清空之前的选项
                    self.active_choices = None;
                },
                OutputEvent::End => el.exit(),

                _ => {}
            }
        }
    }
}

impl Screen for InGameScreen {
    fn update(
        &mut self,
        dt: f32,
        ctx: &mut Ctx,
        el: &ActiveEventLoop,
        assets: &AssetManager,
        audio: &mut AudioPlayer
    ) -> ScreenTransition {

        // 1. 驱动 VM 执行脚本
        let mut waiting = false;
        for _ in 0..100 {
            waiting = self.driver.step(ctx);
            if waiting { break; }
        }

        // 2. 处理产生的事件 (音频播放、立绘移动)
        self.process_output_events(ctx, el, assets, audio);

        // 3. 更新动画状态
        self.animator.update(dt);

        ScreenTransition::None
    }

    fn draw(&mut self, ui: &mut UiDrawer, painter: &mut Painter, rect: Rect, ctx: &mut Ctx) {
        // ============================
        // 1. 绘制场景 (Layer 0)
        // ============================
        // 调用 Painter 画背景和立绘。
        // Painter 应该只需要知道在这个 rect 范围内画画
        painter.paint(ui, ctx, &self.animator, (rect.w, rect.h));

        // ============================
        // 2. 布局 UI (Rect Cut)
        // ============================
        let (bottom_area, _game_area) = rect.split_bottom(280.0); // 底部 300px 给对话框

        // ============================
        // 3. 绘制对话框 (Layer 1)
        // ============================
        if let Some(last_dialogue) = ctx.dialogue_history.last() {
            // 背景板
            Panel::new()
                .gradient(
                    GradientDirection::Vertical,
                    Color::rgba(20, 60, 70, 220),    // 深青色
                    Color::rgba(40, 180, 200, 180)   // 亮青色
                )
                .show(ui, bottom_area);

            let dialogue_area = bottom_area.shrink(30.0);

            let (_, left) = dialogue_area.split_left(300.0);
            let (_, content_area) = left.split_right(300.0);

            let (name_rect, text_rect) = content_area.split_top(50.0);
            // 名字 (如果有)
            if let Some(name) = &last_dialogue.speaker {
                // 有名字：在头部区域画名字
                let name_text = format!("【{}】", name);
                Label::new(&name_text)
                    .size(32.0)
                    .color(Color::rgb(255, 230, 200)) // 米黄色
                    .align(Alignment::Start)
                    .show(ui, name_rect);
            }

            Label::new(&last_dialogue.text)
                .size(26.0)
                .color(Color::WHITE)
                .align(Alignment::Start)
                .show(ui, text_rect.shrink(10.0));

            let icon_x = bottom_area.x + bottom_area.w - 200.0;
            let icon_y = bottom_area.y + bottom_area.h - 60.0;

            ui.draw_circle((icon_x, icon_y), 10.0, Color::rgba(255, 255, 255, 150));
        }

        // ============================
        // 4. 绘制选项 (Layer 2 - Modal)
        // ============================
        if let Some((title, options)) = &self.active_choices {
            // 全屏半透明遮罩
            Panel::new()
                .color(Color::rgba(0, 0, 0, 150))
                .show(ui, rect);

            // 居中菜单
            let menu_area = rect.center(600.0, 500.0);
            let (header, mut body) = menu_area.split_top(80.0);

            if let Some(t) = title {
                Label::new(t).size(36.0).show(ui, header);
            }

            for (idx, txt) in options.iter().enumerate() {
                let (btn, rest) = body.split_top(80.0);
                body = rest;

                if Button::new(txt).show(ui, btn.shrink(10.0)) {
                    self.driver.feed(ctx, InputEvent::ChoiceMade { index: idx });
                    // 点击后清空 active_choices 由 process_output_events 决定
                    // 但这里为了即时反馈可以先置空，或者等待下一帧更新
                }
            }
            // 选项模式下，阻断后续点击
            return;
        }

        // ============================
        // 5. 点击继续逻辑 (Invisible Layer)
        // ============================
        // 只有当鼠标点击了整个区域，且没有被上面的 Button 拦截时，才触发
        if ui.interact(rect).is_clicked() {
            self.driver.feed(ctx, InputEvent::Continue);
        }
    }
}