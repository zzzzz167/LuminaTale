local log = {}

local function clean_path(src)
    if not src then return "[Lua]" end
    -- 2. 截取 "//" 之后的部分
    -- 找到 "//" 的结束位置
    local _, end_idx = src:find("//")
    if end_idx then
        -- 取 // 后面的子串 (比如 system/layouts.lua)
        return src:sub(end_idx + 1)
    end

    return src
end

local function get_caller_info()
    -- debug.getinfo(3) 通常是调用 log.info 的那个函数
    -- "S" 代表 source (文件名), "l" 代表 currentline (行号)
    local info = debug.getinfo(3, "Sl")
    if info then
        local short_path = clean_path(info.short_src)
        return string.format("[%s:%d]", short_path, info.currentline)
    end
    return "[Lua]"
end

local function dump_val(val)
    if type(val) == "table" then
        return "{...table...}" -- 简化输出，防止刷屏
    end
    return tostring(val)
end

local function format_msg(args)
    local str = ""
    for _, v in ipairs(args) do str = str .. dump_val(v) .. " " end
    return str
end

function log.info(...)
    local prefix = "[Lua]" .. get_caller_info()
    -- 调用 Rust 的 info!
    _rust_log.info(prefix .. " " .. format_msg({...}))
end

function log.warn(...)
    local prefix = "[Lua]" ..  get_caller_info()
    -- 调用 Rust 的 warn!
    _rust_log.warn(prefix .. " " .. format_msg({...}))
end

function log.error(...)
    local prefix = "[Lua]" .. get_caller_info()
    -- 调用 Rust 的 error!
    _rust_log.error(prefix .. " " .. format_msg({...}))
end

function log.debug(...)
    local prefix = "[Lua]" .. get_caller_info()
    -- 调用 Rust 的 debug!
    _rust_log.debug(prefix .. " " .. format_msg({...}))
end


return log