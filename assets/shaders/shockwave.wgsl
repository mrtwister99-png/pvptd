// Razova vlna (vybuch, pulz Geyseru, past Spikes): kruh, ktery se rozsiri a vybledne.
// Zacatek a delka jsou v uniformu, takze se nic nemusi menit kazdy snimek.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct WaveParams {
    color: vec4<f32>,
    extra: vec4<f32>, // x = cas zacatku (globals.time), y = delka v sekundach
}
@group(2) @binding(0) var<uniform> params: WaveParams;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let k = (globals.time - params.extra.x) / max(params.extra.y, 0.01); // 0 -> 1 behem vlny
    if (k < 0.0 || k > 1.0) { discard; }
    let p = in.uv * 2.0 - 1.0;
    let d = length(p);
    if (d > 1.0) { discard; }
    let r = 1.0 - pow(1.0 - k, 3.0);              // rychly start, pomaly dojezd
    let thick = 0.10 + 0.25 * (1.0 - k);
    let ring = smoothstep(r - thick, r, d) * (1.0 - smoothstep(r, r + 0.04, d));
    let flash = smoothstep(r, 0.0, d) * (1.0 - k) * 0.45; // zablesk uvnitr na zacatku
    let fade = 1.0 - k;
    let a = (ring * fade + flash * fade) * params.color.a;
    let col = mix(params.color.rgb, vec3<f32>(1.0), flash);
    return vec4<f32>(col, clamp(a, 0.0, 1.0));
}
