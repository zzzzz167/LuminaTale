# Lumina Tale

现代视觉小说引擎框架 | 基于Rust构建

## ✨ 特性

⚡️ 多后端渲染支持（Skia/TUI）

📚 接近自然语言的Viviscript脚本

🦀 RUST语言构建

🔧 TOML配置驱动

📦 模块化架构设计

🦭 Lua语言支持

## 🚀 快速开始

```bash
# 克隆仓库
git clone https://github.com/zzzzz167/LuminaTale.git
cd lumina-tale

# 运行示例游戏
cargo run --package example-game
```
---

## 开发路线图

### MPV:

- ✅ VVS v0.1语法
- ✅ 词法/语法分析 + AST
- ✅ Executor 执行流程
- ✅ 存档/读档
- ✅ 终端渲染器 (ratatui)
- ✅ 配置系统 + 日志
- ✅ README
- ❌ Lua能力支持
- ❌ 完整示例游戏

### V0.2
- ❌ VVS更多语法支持
- ❌ GUI开发
- ❌ 音频播放
- ❌ 资源加载与加载预测

---

## 👀 项目长期愿景

### 引擎定位
下一代开源跨平台Galgame引擎，致力于:

🕹️ 降低非程序员创作门槛

🚀 提供媲美商业引擎的性能表现

🌐 实现真正的全平台原生支持

### 核心目标
1. **现代化渲染架构**
    - 基于Skia+Vulkan的混合渲染管线
    - 自动适配2D/2.5D视觉表现需求
    - 支持动态光影与粒子特效

2. **智能脚本系统(Viviscript)**
    - 类自然语言的剧本编写语法
    - 内嵌Lua扩展接口
    - 实时热重载调试

3. **跨平台能力**
    - 桌面端：Windows/macOS/Linux
    - 移动端：iOS/Android

4. **创作者生态**
    - 开箱即用的创作

### 中期目标

#### V1.0
- 语义版本发布
- 自动跨平台打包
- 运行时性能分析
- 官方模板仓库 & 教程
- Steam SDK 集成

---
## 🤝 贡献
欢迎 PR、Issue、讨论！

---
**Enjoy your story!**