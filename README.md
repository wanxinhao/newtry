# 万物共生 - 生命演化模拟

一个基于 Rust + WebAssembly 构建的生态系统演化模拟应用，在浏览器中实时模拟多种物种的相互作用与进化过程。

## 🌟 项目亮点

- **高性能技术栈**：Rust + WebAssembly，运行效率接近原生应用
- **完整生态系统**：包含植物、草食动物、肉食动物、分解者四类物种的完整食物链
- **智能演化机制**：物种会根据环境压力和生存竞争自动进化适应
- **沉浸式界面**：深色玻璃拟态设计，实时 Canvas 渲染生态动态
- **实时监控面板**：左侧物种信息、右侧环境统计、底部事件日志，全方位追踪生态变化
- **参数可调**：支持调整模拟速度和各项生态参数

## 🎮 使用场景

1. **教育演示**：用于生物学课堂展示生态系统运作和自然选择原理
2. **科学研究**：模拟不同环境参数对物种演化的影响
3. **算法验证**：测试复杂自适应系统和多智能体交互算法
4. **科普互动**：向公众展示生命演化的魅力和复杂性
5. **娱乐体验**：观察虚拟生命在数字世界中诞生、繁衍、进化

## 🚀 快速开始

### 运行已构建版本

项目已预编译完成，直接启动 HTTP 服务器即可运行：

```bash
cd ecosystem-sim
python3 -m http.server 8080
```

然后在浏览器中打开：http://localhost:8080

### 重新构建项目

如果你需要修改源代码并重新构建：

```bash
# 安装依赖
cargo install wasm-bindgen-cli

# 编译 Rust WASM
cargo build --target wasm32-unknown-unknown --release

# 生成 JS 绑定
wasm-bindgen target/wasm32-unknown-unknown/release/ecosystem_sim.wasm --out-dir pkg --web
```

或者使用构建脚本：

```bash
./build.sh
```

## 📁 项目结构

```
ecosystem-sim/
├── src/                    # Rust 源代码
│   ├── render/             # 渲染模块
│   ├── simulation/         # 模拟逻辑
│   │   ├── config.rs       # 配置参数
│   │   ├── environment.rs  # 环境系统
│   │   ├── organism.rs     # 生物个体
│   │   ├── species.rs      # 物种定义
│   │   └── world.rs        # 世界管理
│   └── lib.rs              # 主入口
├── pkg/                    # WASM 绑定文件（编译产物）
├── index.html              # 前端界面
├── build.sh                # 构建脚本
├── Cargo.toml              # Rust 依赖配置
└── README.md               # 项目说明
```

## 🎨 界面功能

| 区域 | 功能 |
|------|------|
| **顶部栏** | 标题、控制按钮、速度调节 |
| **左侧面板** | 物种列表、种群数量、进化状态 |
| **中央画布** | 生态系统实时渲染 |
| **右侧面板** | 环境统计、事件日志 |
| **底部状态栏** | 运行状态、模拟信息 |

## 📊 物种类型

- 🌱 **植物** - 生产者，通过光合作用获取能量
- 🟦 **草食动物** - 初级消费者，以植物为食
- 🟥 **肉食动物** - 次级消费者，捕食其他动物
- 🟪 **分解者** - 分解死亡生物，循环物质

## 🛠️ 技术栈

- **Rust** - 核心模拟逻辑
- **WebAssembly** - 高性能跨平台执行
- **wasm-bindgen** - Rust 与 JavaScript 桥接
- **WebGL** - Canvas 实时渲染
- **CSS3** - 现代样式设计

## 📝 License

MIT License
