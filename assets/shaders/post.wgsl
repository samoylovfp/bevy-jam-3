#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

@group(1) @binding(2)
var<uniform> blur_strength: vec4<f32>;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    // Get screen position with coordinates from 0 to 1
    let uv = coords_to_viewport_uv(position.xy, view.viewport);

    let resolution = 2;

    var output_color = vec4<f32>();

    for (var x=-resolution; x<=resolution; x++) {
        for (var y=-resolution; y<=resolution; y++) {
            let px_strength = abs(max(f32(x*2), 1.0) * max(f32(y*2), 1.0));
            output_color += textureSample(texture, our_sampler, uv + vec2<f32>(f32(x)*blur_strength.x, f32(y)*blur_strength.x))
                /px_strength
                /4.0
                /f32(resolution*resolution);
        }
    }

    return output_color;
}
