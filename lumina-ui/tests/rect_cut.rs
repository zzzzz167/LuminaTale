#[cfg(test)]
mod tests {
    use lumina_ui::Rect;

    #[test]
    fn test_cut() {
        let screen = Rect::new(0.0, 0.0, 1000.0, 800.0);

        // 切顶部 100
        let (header, body) = screen.split_top(100.0);
        assert_eq!(header, Rect::new(0.0, 0.0, 1000.0, 100.0));
        assert_eq!(body, Rect::new(0.0, 100.0, 1000.0, 700.0));

        // 从剩下的 body 切左边 200
        let (sidebar, content) = body.split_left(200.0);
        assert_eq!(sidebar, Rect::new(0.0, 100.0, 200.0, 700.0));
        assert_eq!(content, Rect::new(200.0, 100.0, 800.0, 700.0));
    }
}