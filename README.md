# wasm2

WASM WebGPU project (wgpu, WebGL removed). Forward rendering with a minimal G-buffer (color, depth reversed-Z, velocity) for Temporal Anti-Aliasing (TAA), Kawase bloom, and lens/screen pass. Stereo: two viewports to swap chain (TAA/post skipped). Optional geometry: ChunkMesh (greedy-meshed voxels), Line2DStrip, Particles (instanced quads).

## Build

- Install the wasm target: `rustup target add wasm32-unknown-unknown`
- Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).
- Dev build: `./build_dev.sh` → serves `index.htm` with `pkg/` from the repo root.
- Release build: `./build_release.sh`
