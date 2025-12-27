print("⚡ [Boot] Loading Engine Extensions...")
lumina.tween = require "system.tween"
function lumina_update(dt) lumina.tween.update(dt) end

lumina.register_layout("left_spot",  {x=0.25, y=1.0})
lumina.register_layout("right_spot", {x=0.75, y=1.0})
lumina.register_layout("mid_air",    {x=0.5,  y=0.5, anchor_y=0.5})

lumina.register_transition("slide_in_right", {
    duration = 0.6,
    easing = "ease_out",
    props = {
        x = { from = 1920 + 200, to = 1440 }, -- 假设 1440 是 right_spot 的 x 坐标
        alpha = { from = 0.0, to = 1.0 }
    }
})

lumina.tween.register_generator("jelly_pop", function(p)
    -- p 是 0.0 到 1.0 的进度
    -- 算法：超过 1.0 再弹回来 (Overshoot)
    -- 0 -> 1.2 -> 0.9 -> 1.0 (大概这种感觉)
    local scale_val = 1.0
    if p < 1.0 then
        scale_val = math.sin(p * math.pi / 2) + math.sin(p * math.pi * 3) * 0.1 * (1-p)
    end

    return {
        scale = scale_val,
        alpha = p -- 顺便变清晰
    }
end, 0.8)

lumina.tween.register_generator("shake_crazy", function(p)
    local intensity = (1.0 - p) * 20 -- 随时间减弱

    local offset_x = (math.random() * 2 - 1) * intensity
    local offset_y = (math.random() * 2 - 1) * intensity

    return {
        x = 960 + offset_x,  -- 假设在中心 (960, 1080) ? 等等，center通常是 y=1080(脚底)
        -- 注意：如果你的 center 布局 y 是 1080，这里要基于 1080 抖动
        y = 540 + offset_y
    }
end, 0.5)

print("✅ [Boot] Systems Ready.")