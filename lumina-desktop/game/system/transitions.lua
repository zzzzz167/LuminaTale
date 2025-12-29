local reg = lumina.register_transition

reg("slide_in_right", {
    duration = 0.6,
    easing = "ease_out",
    props = {
        x = { from = 1920 + 200, to = 1440 }, -- 假设 1440 是 right_spot 的 x 坐标
        alpha = { from = 0.0, to = 1.0 }
    }
})

reg("dissolve", {
    duration = 0.3,
    easing = "ease_in_out",
    props = {
        alpha = { from = 0.0, to = 1.0 }
    }
})

reg("circle_open", {
    duration = 1.5,
    easing = "ease_in_out",
    mask_img = "rules/circle.png",
    vague = 0.2,
})

lumina.log.info("Transitions loaded.")