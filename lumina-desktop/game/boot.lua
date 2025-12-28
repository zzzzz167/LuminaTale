lumina.tween = require "system.core.tween"
lumina.log = require "system.core.log"
lumina.log.info("⚡ Loading Engine Extensions...")
function lumina_update(dt) lumina.tween.update(dt) end

require "system.layouts"
require "system.transitions"
require "system.effects"

lumina.log.info("Systems Ready.")