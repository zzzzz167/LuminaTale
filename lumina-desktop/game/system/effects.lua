local reg = lumina.tween.register_generator

reg("jelly_pop", function(p)
    local scale_val = 1.0
    if p < 1.0 then
        scale_val = math.sin(p * math.pi / 2) + math.sin(p * math.pi * 3) * 0.1 * (1-p)
    end

    return {
        scale = scale_val,
        alpha = p -- 顺便变清晰
    }
end, 0.8)

reg("shake_crazy", function(p)
    local intensity = (1.0 - p) * 20

    return {
        -- 直接返回偏移量
        offset_x = (math.random() * 2 - 1) * intensity,
        offset_y = (math.random() * 2 - 1) * intensity
    }
end, 0.5)

lumina.log.info("effects loaded.")