Here there be dragons 🦀

This is an early alpha version and my first larger rust project. You are likely to find some very questionable practices.

General idea is that the Rust WASM handles all of the rendering + application logic. If rust needs either data from VSCode or file system access (because it's wasm in a "browser"), a message is passed from `backend.rs -> bridge.js -> VSCode`. Messages coming the other direction are handled in `bridge.js -> wasm.rs`.

Because egui is [immediate mode](https://en.wikipedia.org/wiki/Immediate_mode_GUI) and we're passing async messages back and forth between wasm and VSCode/eth, we track a fair bit of global state in `shared_state.rs`.

`app.rs` is the primary rendering driver via the `update()` function, which is called many times per second.
