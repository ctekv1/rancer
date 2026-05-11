//! Tests for SDL2 window rendering pipeline

#[test]
fn shaders_are_defined() {
    use crate::renderer;
    assert!(!renderer::VERTEX_SHADER.is_empty());
    assert!(!renderer::FRAGMENT_SHADER.is_empty());
}

#[test]
fn vertex_shader_has_version_and_main() {
    use crate::renderer;
    let src = renderer::VERTEX_SHADER;
    assert!(src.contains("#version"));
    assert!(src.contains("main()"));
    assert!(src.contains("gl_Position"));
}

#[test]
fn fragment_shader_has_output_and_texture() {
    use crate::renderer;
    let src = renderer::FRAGMENT_SHADER;
    assert!(src.contains("#version"));
    assert!(src.contains("main()"));
    assert!(src.contains("fragColor") || src.contains("gl_FragColor"));
    assert!(src.contains("u_texture"));
}