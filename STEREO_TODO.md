# Stereo / VR improvements (TODO)

Rendering backend is WebGPU (wgpu). Stereo mode draws two viewports (left/right) to the swap chain; TAA and post are skipped in stereo.

- **Post-processing in stereo**  
  TAA and the blur chain are currently disabled in stereo mode; the final pass composites from the raw color buffer. Re-enable and adapt for stereo later: run TAA per eye (separate history per viewport), run blur per half or with correct boundaries, and/or add a stereo-aware screen pass (e.g. tonemap per view, no cross-eye bloom).

- **Raymarched scene in stereo**  
  Warehouse (raymarching) uses per-eye viewport and matrices; both eyes get the correct perspective. When TAA is re-enabled per eye, confirm depth and motion vectors per viewport.

- **WebXR session integration**  
  The VR button currently only toggles stereo (side-by-side) on the canvas. `xr::request_immersive_session()` is stubbed: it returns an error ("WebGPU XR layer not yet implemented") because the XR layer API is WebGL-based. When the platform supports WebGPU for XR, wire the VR button to request a session and render using `xr::view_states_from_frame` each frame into the XR layer’s framebuffer/viewports.

- **Stereo output on 2D display**  
  For non-headset use, consider anaglyph or other 2D stereo viewing options, or keep the current left/right split so both views are visible.
