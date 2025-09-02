# Oxide Render Engine

Oxide is a **web-based 3D rendering engine** written in **Rust**, powered by **[WGPU](https://github.com/gfx-rs/wgpu)** and **[WebGPU](https://www.w3.org/TR/webgpu/)**.
It’s designed as part of a Bachelor's thesis in **Computer Graphics** and focuses on providing a modern, high-performance rendering pipeline that works natively and in the browser via **[WASM](https://webassembly.org/)**.

---

## ✨ Features

* 🚀 Built with **Rust** and **wgpu** for cross-platform GPU rendering
* 🌐 Compiles to **WebAssembly** for browser-based applications
* 🖼️ Integrated **winit** window handling for native builds
* 🧩 Structured for future scalability (ECS, multi-threaded execution planned)
* 🛠️ Logging with `log::info!` (currently prints pressed keys to the browser console)
* 🔧 Easy development workflow for both **native** and **WASM** targets

---

## 📦 Project Structure

```
oxide/
├── src/               # Engine source code
├── Cargo.toml         # Rust manifest
├── Cargo.lock         # Dependency lock file
├── pkg/               # Generated WASM package (after build, not version controlled)
└── static/            # HTML index file for browser demo
```

---

## 🚀 Getting Started

### **1. Clone the repository**

```bash
git clone https://github.com/SafetImamovic/oxide.git
cd oxide
```

---

### **2. Run natively**

Enable info-level logging and run:

```bash
RUST_LOG=info cargo run
```

This launches the engine in a native window using `winit`.

---

### **3. Build for WebAssembly**

When targeting browsers, randomness support must be configured explicitly for `getrandom`.
Use the following command to build:

```bash
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --target web --no-default-features
```

This generates the `pkg/` folder containing the `.wasm` and JS bindings.
# Oxide Render Engine

A modern, Rust-based rendering engine using wgpu, winit, and egui.

## Features

- Modular render pass system with a flexible render graph
- Support for different fill modes (Fill, Wireframe, Vertex)
- Depth buffer support for 3D rendering
- Camera system with perspective projection
- Cross-platform support (native and WebAssembly)
- Integrated debug UI with egui

## Architecture

The engine is built around these key components:

- **EngineState**: Manages GPU resources and rendering state
- **RenderGraph**: Orchestrates render passes in a flexible pipeline
- **RenderPass**: Interface for implementing different rendering stages
- **PipelineManager**: Handles creation and management of rendering pipelines
- **Camera**: 3D perspective camera system

## Getting Started

```rust
fn main() -> anyhow::Result<()> {
    // Create the engine with default settings
    let engine = oxide::engine::EngineBuilder::new()
        .build()?;

    // Create a runner to execute the engine
    let runner = oxide::engine::EngineRunner::new(engine)?;

    // Run the engine
    runner.run()
}
```

## License

MIT
---

### **4. Serve locally with Python**

To bypass CORS restrictions when testing in the browser:

```bash
python -m http.server 8080
```

Then open [http://localhost:8080/static/](http://localhost:8080/static/) in your browser.

---

## 🧭 Controls

* **Keyboard Input**: Key presses are currently logged to the **browser console** (`log::info!`).
* Escape (`ESC`) closes the window in native builds.

---

## 🔮 Roadmap

* [ ] Basic 3D rendering pipeline
* [ ] Camera controls and transformations
* [ ] Scene graph management
* [ ] Asset loading (models, textures)
* [ ] Multi-threaded task scheduling for physics and async asset loading
* [ ] WebGPU shader management

---

## 🛠️ Development Notes

* Built with Rust **edition 2024**
* Uses:

  * [`wgpu`](https://github.com/gfx-rs/wgpu) – GPU abstraction layer
  * [`winit`](https://github.com/rust-windowing/winit) – Window management
  * [`anyhow`](https://crates.io/crates/anyhow) – Error handling
  * [`log`](https://crates.io/crates/log) + `env_logger` – Logging
  * [`console_log`](https://crates.io/crates/console_log) – Browser console logs for WASM

---

## 📄 License

TODO!


