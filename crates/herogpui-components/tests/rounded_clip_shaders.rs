//! Validate the renderer's actual assembled shaders in normal workspace tests,
//! including on platforms that do not use WGPU for their native window.
#[path = "../../gpui_pre_wgpu/src/shaders.rs"]
mod shaders;

use shaders::WEBGL_SHADERS;

#[test]
fn webgl_surface_clips_do_not_require_the_instance_texture() {
    let module = naga::front::wgsl::parse_str(WEBGL_SHADERS).unwrap();
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .unwrap();
    let entry = module
        .entry_points
        .iter()
        .position(|entry| entry.name == "fs_surface")
        .unwrap();
    let usage = info.get_entry_point(entry);
    let surface = module
        .types
        .iter()
        .find(|(_, ty)| ty.name.as_deref() == Some("SurfaceParams"))
        .unwrap()
        .1;
    assert!(
        matches!(surface.inner, naga::TypeInner::Struct { span: 64, .. }),
        "WebGL uniforms must retain their 16-byte size alignment"
    );
    let mut reads_clips = false;
    for (handle, global) in module.global_variables.iter() {
        if usage[handle].is_empty() {
            continue;
        }
        if let Some(binding) = &global.binding {
            if binding.group == 1 && binding.binding == 0 {
                assert_eq!(
                    global.space,
                    naga::AddressSpace::Uniform,
                    "surface binding 1:0 is a uniform, never the instance texture"
                );
            }
            reads_clips |= binding.group == 3 && binding.binding == 0;
        }
    }
    assert!(
        reads_clips,
        "surface fragments must retain rounded clipping"
    );
}
