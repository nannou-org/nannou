use bevy::render::render_resource::ShaderRef;
use bevy_nannou_derive::shader_model;
use bevy_nannou_draw::render::ShaderModel;

#[shader_model]
struct TestMaterial {}

#[test]
fn test_default_shaders() {
    assert!(matches!(TestMaterial::vertex_shader(), ShaderRef::Default));
    assert!(matches!(
        TestMaterial::fragment_shader(),
        ShaderRef::Default
    ));
}

#[shader_model(vertex = "custom_vertex.wgsl")]
struct TestVertexMaterial {}

#[test]
fn test_custom_vertex_shader() {
    assert!(matches!(
        TestVertexMaterial::vertex_shader(),
        ShaderRef::Path(_)
    ));
    assert!(matches!(
        TestVertexMaterial::fragment_shader(),
        ShaderRef::Default
    ));
}

#[shader_model(fragment = "custom_fragment.wgsl")]
struct TestFragmentMaterial {}

#[test]
fn test_custom_fragment_shader() {
    assert!(matches!(
        TestFragmentMaterial::vertex_shader(),
        ShaderRef::Default
    ));
    assert!(matches!(
        TestFragmentMaterial::fragment_shader(),
        ShaderRef::Path(_)
    ));
}

#[shader_model(vertex = "custom_vertex.wgsl", fragment = "custom_fragment.wgsl")]
struct TestBothMaterial {}

#[test]
fn test_both_custom_shaders() {
    assert!(matches!(
        TestBothMaterial::vertex_shader(),
        ShaderRef::Path(_)
    ));
    assert!(matches!(
        TestBothMaterial::fragment_shader(),
        ShaderRef::Path(_)
    ));
}
