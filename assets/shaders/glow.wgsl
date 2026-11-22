// Zarici strela: bile horke jadro, barevna zare kolem, jemne mihotani.
// Kresli se na ctverec (UV 0..1), strela je uprostred.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct GlowParams {
    color: vec4<f32>, // barva zare (a = sila)
    extra: vec4<f32>, // x = faze mihotani (aby strely neblikaly naraz)
}
@group(2) @binding(0) var<uniform> params: GlowParams;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * 2.0 - 1.0;
    let d = length(p);
    let core = smoothstep(0.32, 0.0, d);                      // horke jadro
    let halo = pow(clamp(1.0 - d, 0.0, 1.0), 2.2);            // mekka zare do stran
    let flicker = 0.85 + 0.15 * sin(globals.time * 28.0 + params.extra.x);
    let a = clamp(core + halo * 0.85 * flicker, 0.0, 1.0) * params.color.a;
    let col = mix(params.color.rgb * 1.4, vec3<f32>(1.0), core * 0.85);
    return vec4<f32>(col, a);
}
