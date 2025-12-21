use skia_safe::Rect;

pub fn compute_layout(
    parent_rect: Rect,
    direction: crate::ui::Direction,
    padding: f32,
    spacing: f32,
    children_count: usize,
    get_child_size: impl Fn(usize, f32, f32) -> (f32, f32),
) -> (Rect, Vec<Rect>){
    let content_rect = Rect::from_xywh(
        parent_rect.x() + padding,
        parent_rect.y() + padding,
        (parent_rect.width() - padding * 2.0).max(0.0),
        (parent_rect.height() - padding * 2.0).max(0.0),
    );

    let mut result_rects = Vec::with_capacity(children_count);
    let mut cursor_x = content_rect.x();
    let mut cursor_y = content_rect.y();

    for i in 0..children_count {
        let (w, h) = get_child_size(i, content_rect.width(), content_rect.height());

        result_rects.push(Rect::from_xywh(cursor_x, cursor_y, w, h));

        match direction {
            crate::ui::Direction::Column => cursor_y += h + spacing,
            crate::ui::Direction::Row => cursor_x += w + spacing,
        }
    }

    (content_rect, result_rects)
}