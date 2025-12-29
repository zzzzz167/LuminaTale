pub mod input;
pub mod types;
pub mod widgets;

pub use types::{Rect, Color, Alignment, Style, Background, Border, GradientDirection, Transform, ShaderSpec};
use input::Interaction;

pub trait UiRenderer {
    /// 万能绘制接口：渲染一个带有背景（纯色/渐变/图片）和边框的矩形
    fn draw_style(&mut self, rect: Rect, style: &Style);

    /// 图片绘制接口:
    /// image_id: 资源 ID (例如 "btn_bg", "character_face")
    /// tint: 染色颜色 (Color::WHITE 为原色)
    fn draw_image(&mut self, image_id: &str, rect: Rect, tint: Color);

    /// 文本绘制
    fn draw_text(&mut self, text: &str, rect: Rect, color: Color, size: f32, align: Alignment, font: Option<&str>);

    /// 绘制圆形
    fn draw_circle(&mut self, center: (f32, f32), radius: f32, color: Color);

    /// 核心交互：查询某个区域的状态 (Hover / Click / Held)
    fn interact(&self, rect: Rect) -> Interaction;

    /// 获取当前鼠标位置 (用于滑块计算数值等)
    fn cursor_pos(&self) -> (f32, f32);

    fn with_transform(&mut self, transform: Transform, f: &mut dyn FnMut(&mut Self));

    fn time(&self) -> f32;

    fn measure_image(&mut self, image_id: &str) -> Option<(f32, f32)>;

    fn draw_shader(&mut self, rect: Rect, spec: ShaderSpec);
}