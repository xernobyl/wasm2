//! WebXR integration for VR: session handling and view extraction.
//!
//! When an immersive VR session is active, view and projection matrices
//! come from the XR frame; otherwise use [`crate::camera::Camera`] or
//! [`crate::stereo_camera::StereoCamera`].
//!
//! Public API and helpers are kept for when the VR button wires to an XR session.

#![allow(dead_code)]

use glam::Mat4;
use wasm_bindgen::JsCast;
use web_sys::{XrFrame, XrReferenceSpace, XrSession, XrView, XrViewerPose};

use crate::stereo_camera::Eye;
use crate::view::ViewState;

/// Checks whether immersive VR is supported (requires user gesture to call from some browsers).
pub async fn is_immersive_vr_supported() -> bool {
    let window = web_sys::window().expect("no window");
    let navigator = window.navigator();
    let xr = navigator.xr();
    let promise = xr.is_session_supported(web_sys::XrSessionMode::ImmersiveVr);
    let js_fut = wasm_bindgen_futures::JsFuture::from(promise);
    match js_fut.await {
        Ok(js_val) => js_val.as_bool().unwrap_or(false),
        Err(_) => false,
    }
}

/// Request an immersive VR session. Call from a user gesture (e.g. button click).
/// Returns session and local reference space for viewer pose.
///
/// WebGPU XR layer integration is not yet implemented; this returns an error.
/// When the platform supports WebGPU for XR, render to the XR layer here.
pub async fn request_immersive_session(
) -> Result<(XrSession, XrReferenceSpace), wasm_bindgen::JsValue> {
    let _ = web_sys::window().expect("no window");
    Err(js_sys::Error::new(
        "WebGPU XR layer not yet implemented; use stereo canvas for now",
    )
    .into())
}

/// Extract left and right [`ViewState`] from an XR frame for the current viewer pose.
/// Returns `None` if the viewer pose is not available (e.g. tracking lost).
pub fn view_states_from_frame(
    frame: &XrFrame,
    reference_space: &XrReferenceSpace,
    viewport_full: (i32, i32),
) -> Option<Vec<ViewState>> {
    let viewer_pose: XrViewerPose = frame.get_viewer_pose(reference_space)?;
    let views = viewer_pose.views();
    let mut states = Vec::with_capacity(2);
    let (fb_w, fb_h) = viewport_full;
    let half_w = fb_w / 2;

    for i in 0..views.length() {
        let view: XrView = views.get(i).dyn_into().ok()?;
        let eye = match view.eye() {
            web_sys::XrEye::Left => Eye::Left,
            web_sys::XrEye::Right => Eye::Right,
            _ => continue,
        };
        let viewport = if eye == Eye::Left {
            (0, 0, half_w, fb_h)
        } else {
            (half_w, 0, fb_w - half_w, fb_h)
        };

        let proj = view.projection_matrix();
        let projection = mat4_from_slice(&proj);

        let transform = view.transform();
        let view_mat_raw = transform.matrix();
        let view_mat = mat4_from_slice(&view_mat_raw);
        let view = view_mat.inverse();

        let view_projection = projection * view;
        let previous = view_projection;
        // XR path: no subpixel jitter; use same matrices for velocity/TAA.
        let state = ViewState::from_matrices(
            view,
            projection,
            previous,
            view_projection,
            previous,
            fovy_from_projection(&projection),
            eye,
            viewport,
        );
        states.push(state);
    }

    if states.is_empty() {
        return None;
    }
    Some(states)
}

fn mat4_from_slice(a: &[f32]) -> Mat4 {
    let mut arr = [0.0; 16];
    let len = a.len().min(16);
    arr[..len].copy_from_slice(&a[..len]);
    Mat4::from_cols_array(&arr)
}

fn fovy_from_projection(p: &Mat4) -> f32 {
    let m = p.to_cols_array();
    let sy = (m[5]).abs();
    if sy > 0.0 {
        2.0 * (1.0 / sy).atan()
    } else {
        std::f32::consts::FRAC_PI_2
    }
}
