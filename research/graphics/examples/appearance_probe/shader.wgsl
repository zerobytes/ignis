struct Globals {
    view_projection: mat4x4<f32>,
    light: vec4<f32>,
    view: vec4<f32>,
    settings: vec4<f32>, // mode, bins, specular exponent, specular amplitude
    jitter: vec4<f32>,
};
struct Lobe { normal_area: vec4<f32>, color: vec4<f32> };
@group(0) @binding(0) var<uniform> g: Globals;
@group(0) @binding(1) var<storage, read> lobes: array<Lobe>;
struct Transition { parent: u32, weight: f32, padding: vec2<u32> };
@group(0) @binding(2) var<storage, read> transitions: array<Transition>;
@group(0) @binding(3) var<storage, read> evaluated: array<vec4<f32>>;
struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};
fn shade(n: vec3<f32>, color: vec3<f32>) -> vec3<f32> {
    let ndl = max(dot(n,g.light.xyz),0.0);
    let h = normalize(g.light.xyz+g.view.xyz);
    let highlight = pow(max(dot(n,h),0.0),g.settings.z)*g.settings.w;
    // A deliberately simple shared response, not a production energy-conserving BRDF.
    return color*(0.03+ndl) + vec3<f32>(highlight*ndl);
}
fn summary(id: u32) -> vec3<f32> {
        var sum = vec3<f32>(0.0);
        var total = 0.0;
        let count = u32(g.settings.y);
        for(var i=0u;i<count;i++) {
            let b = lobes[id*count+i];
            if b.normal_area.w > 0.0 {
                let weight = b.normal_area.w*max(dot(b.normal_area.xyz,g.view.xyz),0.0);
                sum += weight*shade(normalize(b.normal_area.xyz),b.color.xyz);
                total += weight;
            }
        }
        return sum/max(total,1e-12);
}
@vertex fn vs_main(
    @location(0) position: vec3<f32>, @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>, @location(3) tile: u32
) -> Out {
    var out: Out;
    out.position = g.view_projection*vec4<f32>(position,1.0);
    out.position = vec4<f32>(out.position.xy + g.jitter.xy*out.position.w, out.position.zw);
    out.color = shade(normal,color);
    let proxy = (tile & 0x80000000u) != 0u;
    if g.settings.x > 1.5 && (g.settings.x < 3.5 || proxy) {
        out.color = summary(tile & 0x7fffffffu);
    }
    if (g.settings.x > 4.5 && g.settings.x < 5.5) || (g.settings.x > 6.5 && g.settings.x < 7.5) {
        out.color = select(vec3<f32>(0.05,0.5,1.0), vec3<f32>(1.0,0.35,0.03), proxy);
    }
    if g.settings.x > 7.5 {
        var p = position;
        var id = tile & 0x7fffffffu;
        // At most eight levels for the bounded 128x128 hierarchy.
        for (var level = 0u; level < 8u; level++) {
            let state = transitions[id];
            if state.weight > 0.0 {
                if level > 0u || !proxy {
                    out.color = mix(out.color, summary(id), state.weight);
                }
                p.y *= 1.0-state.weight;
            }
            if state.parent == 0xffffffffu { break; }
            id = state.parent;
        }
        out.position = g.view_projection*vec4<f32>(p,1.0);
        out.position = vec4<f32>(out.position.xy + g.jitter.xy*out.position.w, out.position.zw);
    }
    return out;
}
@fragment fn fs_main(in: Out) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color,1.0);
}

// Same node order, mixtures and height arithmetic as mode8; only summary lookup changes.
@vertex fn vs_cached(
    @location(0) position: vec3<f32>, @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>, @location(3) tile: u32
) -> Out {
    var out: Out;
    let proxy = (tile & 0x80000000u) != 0u;
    var id = tile & 0x7fffffffu;
    out.color = shade(normal,color);
    if proxy { out.color = evaluated[id].xyz; }
    var p = position;
    for (var level = 0u; level < 8u; level++) {
        let state = transitions[id];
        if state.weight > 0.0 {
            if level > 0u || !proxy { out.color = mix(out.color, evaluated[id].xyz, state.weight); }
            p.y *= 1.0-state.weight;
        }
        if state.padding.x == 0xffffffffu { break; }
        id = state.padding.x;
    }
    out.position = g.view_projection*vec4<f32>(p,1.0);
    out.position = vec4<f32>(out.position.xy + g.jitter.xy*out.position.w, out.position.zw);
    return out;
}

@vertex fn vs_full(
    @location(0) position: vec3<f32>, @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>, @location(3) tile: u32
) -> Out {
    var out: Out;
    out.position = g.view_projection*vec4<f32>(position,1.0);
    out.position = vec4<f32>(out.position.xy + g.jitter.xy*out.position.w, out.position.zw);
    out.color = shade(normal,color);
    return out;
}
