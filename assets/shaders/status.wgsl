// Zpomaleni (Frost): svitici ledovy obrys kolem creepa. Velikost (= zbyvajici cas) se meni pres scale,
// takze se obrys zmensuje k telu, jak zpomaleni dobiha. Po obvodu obihaji jemne jiskry.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct StatusParams {
    color: vec4<f32>,
    extra: vec4<f32>,
}
@group(2) @binding(0) var<uniform> params: StatusParams;

const TAU: f32 = 6.2831853;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv * 2.0 - 1.0;
    let d = length(p);
    if (d > 1.0) { discard; }
    let ring = exp(-abs(d - 0.9) * 30.0);               // tenky svitici obrys
    let inner = smoothstep(0.55, 0.9, d) * 0.12;        // jemna zare dovnitr
    let ang = atan2(p.y, p.x) / TAU + 0.5;
    let spark = pow(max(sin((ang * 8.0 - globals.time * 0.8) * TAU), 0.0), 8.0); // obihajici jiskry
    let a = (ring * (0.7 + 0.6 * spark) + inner) * params.color.a;
    let col = mix(params.color.rgb, vec3<f32>(1.0), ring * spark * 0.7);
    return vec4<f32>(col, clamp(a, 0.0, 1.0));
}
