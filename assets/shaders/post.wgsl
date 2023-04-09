#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    // Get screen position with coordinates from 0 to 1
    let uv = coords_to_viewport_uv(position.xy, view.viewport);
    let blur_strength = 0.002;

    var output_color = vec4<f32>();
    // 5-point blur
    for (var x=-2; x<=2; x++) {
        for (var y=-2; y<=2; y++) {
            var px_strength = abs(max(f32(x), 1.0) * max(f32(y), 1.0));
            output_color += textureSample(texture, our_sampler, uv + vec2<f32>(f32(x)*blur_strength, f32(y)*blur_strength))/px_strength/20.0;
        }
    }

    return output_color;
}
