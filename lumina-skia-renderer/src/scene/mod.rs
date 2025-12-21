//TODO: 提出DSL草案来替代这里写死的布局
pub mod animator;
pub use animator::SceneAnimator;

use crate::ui_state::UiState;
use lumina_core::renderer::driver::ExecutorHandle;
use lumina_core::Ctx;
use skia_safe::Color;

use crate::ui::{
    Button,
    Checkbox,
    Label,
    Slider,
    UiAction,
    WidgetNode
};

pub enum AppScene {
    MainMenu {
        root: WidgetNode,
    },
    InGame {
        ctx: Ctx,
        driver: ExecutorHandle,
        ui_state: UiState,
    },
    Settings {
        prev_scene: Box<AppScene>,
        root: WidgetNode,
    }
}

impl Default for AppScene {
    fn default() -> Self {
        let root = WidgetNode::column(vec![
            // 标题
            WidgetNode::Label(Label::new("Lumina Tale", 60.0, Color::WHITE)),

            WidgetNode::Spacer(40.0),

            // 按钮组
            WidgetNode::Button(Button::new("Start Game", UiAction::RunScript("StartGame".to_string()))),
            WidgetNode::Button(Button::new("Settings", UiAction::OpenMenu("Settings".to_string()))), // 新增入口
            WidgetNode::Button(Button::new("Quit", UiAction::Quit)),
        ])
            .with_style(|s| {
                // 定义容器样式
                s.width = Some(400.0);
                s.padding = 40.0;
                s.spacing = 20.0;
                s.bg_color = Some(Color::from_argb(200, 30, 30, 30)); // 半透明深色背景
            });
        AppScene::MainMenu { root }
    }
}

impl AppScene {
    /// 创建设置菜单 (工厂方法)
    pub fn new_settings(prev: AppScene) -> Self {
        // --- 构建设置菜单 UI 树 ---
        let root = WidgetNode::column(vec![
            // 顶部标题
            WidgetNode::Label(Label::new("Settings", 40.0, Color::WHITE)),

            WidgetNode::Spacer(30.0),

            // 音量设置区域
            WidgetNode::Label(Label::new("Audio Configuration", 24.0, Color::LIGHT_GRAY)),
            WidgetNode::Slider(Slider::new("BGM Volume", "music", 0.7)),
            WidgetNode::Slider(Slider::new("Voice Volume", "voice", 0.9)),
            WidgetNode::Slider(Slider::new("SE Volume", "sound", 1.0)),

            WidgetNode::Spacer(20.0),

            // 图像设置区域
            WidgetNode::Label(Label::new("Graphics", 24.0, Color::LIGHT_GRAY)),
            WidgetNode::Checkbox(Checkbox::new("Fullscreen", true, UiAction::ToggleConfig("fullscreen"))),
            WidgetNode::Checkbox(Checkbox::new("Skip Unread Text", false, UiAction::ToggleConfig("skip_unread"))),

            WidgetNode::Spacer(40.0),

            // 底部返回按钮
            WidgetNode::Button(Button::new("Back", UiAction::Back)),
        ])
            .with_style(|s| {
                s.width = Some(600.0); // 设置菜单宽一点
                s.padding = 50.0;
                s.spacing = 15.0;
                s.bg_color = Some(Color::from_argb(240, 20, 20, 20)); // 更不透明的背景
            });

        AppScene::Settings {
            prev_scene: Box::new(prev),
            root,
        }
    }
}