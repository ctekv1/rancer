//! GL function loader that bypasses libepoxy
//!
//! Loads OpenGL ES functions directly from libGLESv2.so using libloading,
//! avoiding the libepoxy context detection issue on Wayland.

use std::ffi::c_void;
use std::sync::OnceLock;

/// Cached handle to the GL library
static GL_LIBRARY: OnceLock<Option<libloading::Library>> = OnceLock::new();

/// Library names to try, in order of preference
const GL_LIBRARY_NAMES: &[&str] = &["libGLESv2.so.2", "libGLESv2.so", "libGL.so.1", "libGL.so"];

/// Initialize the GL library handle
pub fn init_gl_library() -> Result<(), String> {
    if GL_LIBRARY.get().is_some() {
        return Ok(());
    }

    let lib = load_gl_library();
    let result = lib.is_some();
    let _ = GL_LIBRARY.set(lib);

    if result {
        Ok(())
    } else {
        Err("Failed to load any GL library".to_string())
    }
}

fn load_gl_library() -> Option<libloading::Library> {
    for name in GL_LIBRARY_NAMES {
        unsafe {
            match libloading::Library::new(name) {
                Ok(lib) => {
                    crate::logger::info(&format!("Loaded GL library: {name}"));
                    return Some(lib);
                }
                Err(_) => continue,
            }
        }
    }
    None
}

/// Create a glow-compatible loader function from the loaded GL library
pub fn create_gl_loader() -> impl FnMut(&str) -> *const c_void {
    move |name: &str| -> *const c_void {
        let lib = match GL_LIBRARY.get() {
            Some(Some(lib)) => lib,
            _ => return std::ptr::null(),
        };

        let name_cstr = match std::ffi::CString::new(name) {
            Ok(s) => s,
            Err(_) => return std::ptr::null(),
        };

        unsafe {
            match lib.get::<*const c_void>(name_cstr.as_bytes()) {
                Ok(sym) => *sym,
                Err(_) => std::ptr::null(),
            }
        }
    }
}
