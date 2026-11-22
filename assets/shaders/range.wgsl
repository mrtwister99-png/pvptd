// Kruh dosahu veze: jemne vyplneny disk, svitici okraj s obihajicimi carkami
// a vlna, ktera se od veze sire k okraji. Kresli se na ctverec, polomer = 1 v UV.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct RangeParams {
    color: vec4<f32>, // barva (podle veze)
    extra: vec4<f32>, // x = tloustka okraje v pomeru k polomeru
}
@group(2) @binding(0) var<uniform> params: RangeParams;

const TAU: f32 = 6.2831853;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * 2.0 - 1.0;
    let d = length(p);
    if (d > 1.0) { discard; }
    let w = max(params.extra.x, 0.004);
    let t = globals.time;

    // okraj: ostry vnejsi kraj, mekky dovnitr
    let edge = smoothstep(1.0 - w * 4.0, 1.0 - w, d) * (1.0 - smoothstep(1.0 - w * 0.5, 1.0, d));
    // carky obihajici po okraji
    let ang = atan2(p.y, p.x) / TAU + 0.5;
    let dash = step(0.45, fract(ang * 36.0 - t * 0.6));
    // vlna od stredu k okraji
    let wave_r = fract(t * 0.45);
    let wave = exp(-abs(d - wave_r) * 40.0) * (1.0 - wave_r) * 0.25;
    // velmi jemna vypln, k okraji silnejsi
    let fill = 0.015 + 0.05 * d * d * d;

    let a = (fill + edge * (0.55 + 0.45 * dash) + wave) * params.color.a;
    let col = mix(params.color.rgb, vec3<f32>(1.0), edge * 0.35);
    return vec4<f32>(col, clamp(a, 0.0, 1.0));
}
