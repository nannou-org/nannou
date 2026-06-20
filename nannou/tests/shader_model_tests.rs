//! Tests for the `#[shader_model]` attribute macro (from `nannou_derive`).
//!
//! These live in `nannou` rather than `nannou_derive` because the macro expands
//! to `::nannou::prelude::*` (and `::bevy::*`) paths, so the generated code only
//! resolves where `nannou` is in scope. Keeping them here avoids `nannou_derive`
//! having to dev-depend on `nannou` - a cycle that can't be published.

use nannou::prelude::ShaderRef;
use nannou::prelude::render::ShaderModel;
use nannou::prelude::shader_model;

#[shader_model]
struct TestShaderModel {}

#[test]
fn test_default_shaders() {
    assert!(matches!(
        TestShaderModel::vertex_shader(),
        ShaderRef::Default
    ));
    assert!(matches!(
        TestShaderModel::fragment_shader(),
        ShaderRef::Default
    ));
}

#[shader_model(vertex = "custom_vertex.wgsl")]
struct TestVertexShaderModel {}

#[test]
fn test_custom_vertex_shader() {
    assert!(matches!(
        TestVertexShaderModel::vertex_shader(),
        ShaderRef::Path(_)
    ));
    assert!(matches!(
        TestVertexShaderModel::fragment_shader(),
        ShaderRef::Default
    ));
}

#[shader_model(fragment = "custom_fragment.wgsl")]
struct TestFragmentShaderModel {}

#[test]
fn test_custom_fragment_shader() {
    assert!(matches!(
        TestFragmentShaderModel::vertex_shader(),
        ShaderRef::Default
    ));
    assert!(matches!(
        TestFragmentShaderModel::fragment_shader(),
        ShaderRef::Path(_)
    ));
}

#[shader_model(vertex = "custom_vertex.wgsl", fragment = "custom_fragment.wgsl")]
struct TestBothShaderModel {}

#[test]
fn test_both_custom_shaders() {
    assert!(matches!(
        TestBothShaderModel::vertex_shader(),
        ShaderRef::Path(_)
    ));
    assert!(matches!(
        TestBothShaderModel::fragment_shader(),
        ShaderRef::Path(_)
    ));
}
