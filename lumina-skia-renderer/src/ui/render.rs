use skia_safe::Canvas;
use skia_safe::textlayout::FontCollection;
use crate::core::AssetManager;

pub struct RenderContext<'a> {
    pub canvas: &'a Canvas,
    pub fonts: &'a FontCollection,
    pub assets: &'a mut AssetManager,
}

pub trait WidgetRender {
    fn render(&self, ctx: &mut RenderContext);
}