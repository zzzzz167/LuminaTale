local tween = {active = {}, generators = {}}

function tween.register_generator(name, fn, duration)
    tween.generators[name] = {fn=fn, duration = duration or 1.0}
    lumina.mark_as_dynamic(name)
end

function tween.run_dynamic(name, target)
    local gen = tween.generators[name]
    if gen then
        -- 启动一个自定义动画
        tween.custom(target, gen.duration, gen.fn)
    end
end

function tween.update(dt)
    for i = #tween.active, 1, -1 do
        local t = tween.active[i]
        t.time = t.time + dt
        local p = math.min(t.time / t.duration, 1.0)
        local props = t.update_fn(p) -- 调用数学函数
        lumina.transform(t.target, props, 0) -- 瞬移
        if p >= 1.0 then table.remove(tween.active, i) end
    end
end

function tween.custom(target, duration, fn)
    table.insert(tween.active, {target=target, duration=duration, update_fn=fn, time=0})
end

return tween