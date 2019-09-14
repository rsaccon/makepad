#![allow(dead_code)]

#[cfg(any(target_os = "linux"))]
mod cx_linux;
#[cfg(target_os = "linux")]
mod cx_opengl;
#[cfg(target_os = "linux")]
mod cx_xlib;

#[cfg(target_os = "macos")]
mod cx_cocoa;
#[cfg(target_os = "ios")]
mod cx_cocoa_ios;
#[cfg(target_os = "windows")]
mod cx_dx11;
#[cfg(target_os = "windows")]
mod cx_hlsl;
#[cfg(any(target_os = "ios"))]
mod cx_ios;
#[cfg(any(target_os = "macos"))]
mod cx_macos;
mod cx_metal;
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod cx_metalsl;
#[cfg(any(target_os = "windows"))]
mod cx_win10;
#[cfg(target_os = "windows")]
mod cx_win32;

#[cfg(target_arch = "wasm32")]
mod cx_wasm32;
#[cfg(target_arch = "wasm32")]
mod cx_webgl;

#[cfg(any(target_arch = "wasm32", target_os = "linux"))]
mod cx_glsl;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod cx_desktop;

#[cfg(target_os = "ios")]
mod cx_tablet;

// shared modules
#[macro_use]
mod cx;
mod animator;
mod area;
mod blit;
mod colors;
mod cx_cursor;
mod cx_fonts;
mod cx_pass;
mod cx_shader;
mod cx_texture;
mod cx_turtle;
mod cx_view;
mod cx_window;
mod elements;
mod events;
mod math;
mod quad;
mod shadergen;
mod text;

pub use crate::blit::*;
pub use crate::cx::*;
pub use crate::elements::*;
pub use crate::quad::*;
pub use crate::text::*;
