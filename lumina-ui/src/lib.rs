pub mod input;
pub mod types;
pub mod widgets;

pub use types::{Rect, Color, Alignment};
use input::Interaction;

pub trait UiRenderer {
    /// 绘制实心矩形
    fn draw_rect(&mut self, rect: Rect, color: Color);

    /// 绘制空心矩形（描边）
    fn draw_border(&mut self, rect: Rect, color: Color, width: f32);

    /// 绘制文字
    fn draw_text(&mut self, text: &str, rect: Rect, color: Color, size: f32);

    /// 绘制圆形
    fn draw_circle(&mut self, center: (f32, f32), radius: f32, color: Color);

    /// 核心交互：查询某个区域的状态
    fn interact(&self, rect: Rect) -> Interaction;

    /// 获取当前鼠标位置 (用于滑块计算数值)
    fn cursor_pos(&self) -> (f32, f32);
}