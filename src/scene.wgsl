struct Globals {
    view_projection: mat4x4<f32>,
    shadow_projection: mat4x4<f32>,
    light: vec4<f32>,
    shadow_settings: vec4<f32>,
    view_position: vec4<f32>,
    environment: vec4<f32>,
    atmosphere: vec4<f32>,
    atmosphere_sun: vec4<f32>,
    atmosphere_right: vec4<f32>,
    atmosphere_up: vec4<f32>,
    atmosphere_forward: vec4<f32>,
    atmosphere_sky: vec4<f32>,
    atmosphere_base: vec4<f32>,
    atmosphere_detail: vec4<f32>,
};
@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var shadow_map: texture_depth_2d;
@group(0) @binding(2) var shadow_sampler: sampler_comparison;
struct DetailSegment { start: vec4<f32>, end: vec4<f32>, };
struct SurfacePattern {
    frequency: vec4<f32>, low: vec4<f32>, high: vec4<f32>, control: vec4<f32>, response: vec4<f32>,
};
struct SurfaceMaterial {
    frequency: vec4<f32>,
    low: vec4<f32>,
    high: vec4<f32>,
    control: vec4<f32>,
    response: vec4<f32>,
    detail_tint: vec4<f32>,
    detail_control: vec4<f32>,
    detail_domain: vec4<f32>,
    segments: array<DetailSegment, 32>,
    secondary: SurfacePattern,
};
struct MaterialTable { items: array<SurfaceMaterial>, };
@group(0) @binding(3) var<storage, read> materials: MaterialTable;
@group(0) @binding(4) var environment_specular: texture_cube<f32>;
@group(0) @binding(5) var environment_diffuse: texture_cube<f32>;
@group(0) @binding(6) var environment_dfg: texture_2d<f32>;
@group(0) @binding(7) var environment_sampler: sampler;
struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) rest: vec3<f32>,
    @location(4) @interpolate(flat) material: u32,
    @location(5) material_projection_weights: vec3<f32>,
};
@vertex fn vs_main(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) color: vec3<f32>,
                  @location(3) rest: vec3<f32>, @location(4) material: u32,
                  @location(5) material_projection_weights: vec3<f32>) -> VertexOut {
    var out: VertexOut;
    out.clip = globals.view_projection * vec4<f32>(position, 1.0);
    out.world = position;
    out.normal = normal;
    out.color = color;
    out.rest = rest;
    out.material = material;
    out.material_projection_weights = material_projection_weights;
    return out;
}
@vertex fn vs_shadow(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return globals.shadow_projection * vec4<f32>(position, 1.0);
}
fn visibility(world: vec3<f32>) -> f32 {
    if globals.shadow_settings.x < 0.5 { return 1.0; }
    let p = globals.shadow_projection * vec4<f32>(world, 1.0);
    let ndc = p.xyz / p.w;
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || ndc.z < 0.0 || ndc.z > 1.0 {
        return 1.0;
    }
    var sum = 0.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let offset = vec2<f32>(f32(x), f32(y)) * globals.shadow_settings.z;
            sum += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + offset, ndc.z - globals.shadow_settings.y);
        }
    }
    return sum / 9.0;
}
fn hash_cell(p: vec3<i32>, seed: u32) -> f32 {
    let q = bitcast<vec3<u32>>(p);
    var h = q.x * 1597334677u ^ q.y * 3812015801u ^ q.z * 2798796415u ^ seed;
    h = (h ^ (h >> 16u)) * 2246822519u;
    h = (h ^ (h >> 13u)) * 3266489917u;
    h = h ^ (h >> 16u);
    return f32(h & 16777215u) / 16777215.0;
}
fn value_noise(p: vec3<f32>, seed: u32) -> f32 {
    let base = vec3<i32>(floor(p));
    let f = fract(p);
    let w = f * f * (3.0 - 2.0 * f);
    let a = mix(hash_cell(base, seed), hash_cell(base + vec3<i32>(1,0,0), seed), w.x);
    let b = mix(hash_cell(base + vec3<i32>(0,1,0), seed), hash_cell(base + vec3<i32>(1,1,0), seed), w.x);
    let c = mix(hash_cell(base + vec3<i32>(0,0,1), seed), hash_cell(base + vec3<i32>(1,0,1), seed), w.x);
    let d = mix(hash_cell(base + vec3<i32>(0,1,1), seed), hash_cell(base + vec3<i32>(1,1,1), seed), w.x);
    return mix(mix(a,b,w.y), mix(c,d,w.y), w.z);
}
fn cellular_noise(p: vec3<f32>, seed: u32) -> f32 {
    let base = vec3<i32>(floor(p));
    var first = 100.0;
    var second = 100.0;
    for (var z = -1; z <= 1; z++) {
        for (var y = -1; y <= 1; y++) {
            for (var x = -1; x <= 1; x++) {
                let cell = base + vec3<i32>(x,y,z);
                let center = vec3<f32>(cell) + vec3<f32>(hash_cell(cell, seed), hash_cell(cell, seed+11u), hash_cell(cell, seed+23u));
                let distance = length(p - center);
                if distance < first { second = first; first = distance; }
                else if distance < second { second = distance; }
            }
        }
    }
    let plate = smoothstep(0.015, 0.15, second - first);
    return plate * (0.75 + 0.25 * value_noise(p * 0.35, seed+97u));
}
fn pattern_field(p: vec3<f32>, footprint: f32, control: vec4<f32>, response: vec4<f32>) -> f32 {
    let seed = u32(control.w);
    var coordinate = p;
    if control.y > 0.0 {
        let low = p * 0.08;
        coordinate += control.y * (vec3<f32>(value_noise(low, seed+11u),
            value_noise(low, seed+23u), value_noise(low, seed+37u)) - 0.5);
    }
    var value = 0.0;
    var weight = 0.5;
    var total = 0.0;
    var frequency = 1.0;
    for (var octave = 0u; octave < 4u; octave++) {
        if octave >= u32(control.z) { break; }
        let coverage = 1.0 - smoothstep(0.25, 0.75, footprint * frequency * (1.0 + control.y));
        var n = 0.5;
        if coverage > 0.0 {
            if response.y > 1.5 { n = cellular_noise(coordinate * frequency, seed + octave * 1013u); }
            else {
                n = value_noise(coordinate * frequency, seed + octave * 1013u);
                if response.y > 0.5 { n = 1.0 - abs(n * 2.0 - 1.0); }
            }
        }
        value += weight * mix(0.5, n, coverage);
        total += weight;
        weight *= 0.5;
        frequency *= 2.0;
    }
    return clamp((value / max(total, 0.0001) - 0.5) * response.z + 0.5, 0.0, 1.0);
}
// All coordinates are authored in the existing rest-material domain.
fn detail_coordinate(rest: vec3<f32>, plane: f32) -> vec2<f32> {
    if plane < 0.5 { return rest.xy; }
    if plane < 1.5 { return rest.xz; }
    return rest.yz;
}
fn surface_detail(q: vec2<f32>, footprint: f32, material: SurfaceMaterial) -> f32 {
    var local = q;
    if material.detail_domain.x > 0.0 {
        local = fract(q / material.detail_domain.xy) * material.detail_domain.xy;
    }
    var field = 0.0;
    for (var i = 0u; i < 32u; i++) {
        if i >= u32(material.detail_control.z) { break; }
        let s = material.segments[i];
        let delta = s.end.xy - s.start.xy;
        let t = clamp(dot(local - s.start.xy, delta) / max(dot(delta, delta), 0.000000000001), 0.0, 1.0);
        let radius = mix(s.start.z, s.end.z, t);
        let distance = length(local - (s.start.xy + t * delta));
        let aa = max(footprint * 0.5, 0.0000001);
        let coverage = 1.0 - smoothstep(0.25, 0.75, footprint / max(2.0 * radius, 0.0000001));
        var cross_section = 1.0 - smoothstep(radius-aa, radius+aa, distance);
        if material.detail_domain.z > 0.5 {
            cross_section = 1.0 - smoothstep(0.0, radius, distance);
        }
        field = max(field, cross_section * coverage);
    }
    return field;
}
fn projection_blend(weights: vec3<f32>, sharpness: f32) -> vec3<f32> {
    let p = pow(max(weights, vec3<f32>(0.0)), vec3<f32>(sharpness * 0.5));
    let total = p.x + p.y + p.z;
    if total <= 0.00000001 { return vec3<f32>(1.0 / 3.0); }
    return p / total;
}
// Legacy diffuse removal/white specular plus Schlick weight, in Lambert units.
fn dielectric(normal: vec3<f32>, light: vec3<f32>, world: vec3<f32>, roughness: f32, f0: f32) -> vec3<f32> {
    let to_view = globals.view_position.xyz - world;
    let no_l = dot(normal, light);
    if f0 <= 0.0 || no_l <= 0.0 || dot(to_view, to_view) < 0.00000001 { return vec3<f32>(0.0); }
    let view = normalize(to_view);
    let no_v = dot(normal, view);
    let half_sum = light + view;
    if no_v <= 0.0 || dot(half_sum, half_sum) < 0.00000001 { return vec3<f32>(0.0); }
    let half_vector = normalize(half_sum);
    let no_h = clamp(dot(normal, half_vector), 0.0, 1.0);
    let vo_h = clamp(dot(view, half_vector), 0.0, 1.0);
    let alpha = roughness * roughness;
    let a2 = alpha * alpha;
    let d_base = no_h * no_h * (a2 - 1.0) + 1.0;
    let distribution = a2 / max(3.14159265 * d_base * d_base, 0.00000001);
    let masking = 0.5 / max(no_l * sqrt(no_v*no_v * (1.0-a2) + a2)
        + no_v * sqrt(no_l*no_l * (1.0-a2) + a2), 0.000001);
    let one_minus = 1.0 - vo_h;
    let fresnel = f0 + (1.0-f0) * one_minus*one_minus*one_minus*one_minus*one_minus;
    return vec3<f32>(fresnel * no_l, 3.14159265 * distribution * masking * fresnel * no_l,
        one_minus*one_minus*one_minus*one_minus*one_minus);
}
// Distant split-sum probe; no visibility from the local directional shadow map.
fn environment_light(base: vec3<f32>, normal: vec3<f32>, world: vec3<f32>, material: SurfaceMaterial) -> vec3<f32> {
    let metallic = material.high.w;
    let f0 = material.response.w;
    var result = vec3<f32>(0.0);
    if metallic < 1.0 {
        var irradiance = textureSampleLevel(environment_diffuse, environment_sampler, normal, 0.0).rgb;
        if material.response.x > 0.0 {
            irradiance += material.response.x * textureSampleLevel(environment_diffuse, environment_sampler, -normal, 0.0).rgb;
        }
        result = base * irradiance * ((1.0-f0) * (1.0-metallic));
    }
    if f0 > 0.0 || metallic > 0.0 {
        let to_view = globals.view_position.xyz-world;
        if dot(to_view,to_view) > 0.00000001 {
            let view = normalize(to_view);
            let no_v = dot(normal,view);
            if no_v > 0.0 {
                let roughness = material.low.w;
                let reflection = reflect(-view,normal);
                let radiance = textureSampleLevel(environment_specular, environment_sampler, reflection, roughness*globals.environment.y).rgb;
                let coordinate = vec2<f32>((clamp(no_v,0.001,1.0)-0.001)/0.999, (roughness-0.2)/0.8);
                let packed = textureSampleLevel(environment_dfg, environment_sampler, (coordinate*63.0+0.5)/64.0, 0.0);
                let weights = vec2<f32>(256.0/257.0,1.0/257.0);
                let ab = vec2<f32>(dot(packed.rg,weights),dot(packed.ba,weights));
                var dielectric_color = vec3<f32>(0.0);
                if f0 > 0.0 {dielectric_color = vec3<f32>(f0*ab.x+ab.y);}
                let metal_color = clamp(base,vec3<f32>(0.0),vec3<f32>(1.0))*ab.x+vec3<f32>(ab.y);
                result += radiance * mix(dielectric_color,metal_color,metallic);
            }
        }
    }
    return result;
}
// Known at pipeline compilation; enabled only when the asset has no procedural channels.
override uniform_material: bool = false;
@fragment fn fs_main(in: VertexOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let material = materials.items[in.material];
    let px = dpdx(in.world);
    let py = dpdy(in.world);
    var tint = vec3<f32>(1.0);
    var hx = 0.0;
    var hy = 0.0;
    if !uniform_material {
        let p = in.rest * material.frequency.xyz;
        // Derivatives remain in uniform control flow; neutral materials skip noise.
        let footprint = max(length(dpdx(p)), length(dpdy(p)));
        let secondary_p = in.rest * material.secondary.frequency.xyz;
        let secondary_footprint = max(length(dpdx(secondary_p)), length(dpdy(secondary_p)));
        let detail_q = detail_coordinate(in.rest, material.detail_control.y);
        let detail_footprint = max(length(dpdx(detail_q)), length(dpdy(detail_q)));
        let rest_dx = dpdx(in.rest);
        let rest_dy = dpdy(in.rest);
        let footprint_xy = max(length(rest_dx.xy), length(rest_dy.xy));
        let footprint_zy = max(length(rest_dx.zy), length(rest_dy.zy));
        let footprint_xz = max(length(rest_dx.xz), length(rest_dy.xz));
        var height = 0.0;
        var projected = vec3<f32>(0.0);
        var projection_weights = vec3<f32>(1.0 / 3.0);
        if material.frequency.w > 0.5 {
            let field = pattern_field(p, footprint, material.control, material.response);
            tint = mix(material.low.xyz, material.high.xyz, field);
            height = field * material.control.x;
        }
        if material.secondary.frequency.w > 0.5 {
            let field = pattern_field(secondary_p, secondary_footprint, material.secondary.control, material.secondary.response);
            tint *= mix(material.secondary.low.xyz, material.secondary.high.xyz, field);
            height += field * material.secondary.control.x;
        }
        if material.detail_control.z > 0.0 {
            if material.detail_control.w <= 0.0 {
                // Keep the planar path's arithmetic and authored coordinates intact.
                let detail = surface_detail(detail_q, detail_footprint, material);
                tint *= mix(vec3<f32>(1.0), material.detail_tint.xyz, detail * material.detail_tint.w);
                height += detail * material.detail_control.x;
            } else {
                // Weights order is the material normal axis: X->ZY, Y->XZ, Z->XY.
                projected = vec3<f32>(
                    surface_detail(in.rest.zy, footprint_zy, material),
                    surface_detail(in.rest.xz, footprint_xz, material),
                    surface_detail(in.rest.xy, footprint_xy, material));
                projection_weights = projection_blend(in.material_projection_weights, material.detail_control.w);
                let detail = dot(projection_weights, projected);
                tint *= mix(vec3<f32>(1.0), material.detail_tint.xyz, detail * material.detail_tint.w);
            }
        }
        // Derivatives execute after reconvergence, including mixed material quads.
        let projected_dx = dpdx(projected);
        let projected_dy = dpdy(projected);
        hx = dpdx(height);
        hy = dpdy(height);
        if material.detail_control.w > 0.0 {
            // Frozen-weight screen covectors, not a derivative of the blended tint.
            hx += material.detail_control.x * dot(projection_weights, projected_dx);
            hy += material.detail_control.x * dot(projection_weights, projected_dy);
        }
    }
    var normal = normalize(in.normal);
    if material.response.x > 0.0 && !front { normal = -normal; }
    let r1 = cross(py, normal);
    let r2 = cross(normal, px);
    let determinant = dot(px, r1);
    if abs(determinant) > 0.0000000001 {
        normal = normalize(abs(determinant) * normal - sign(determinant) * (hx*r1 + hy*r2));
    }
    let light = -normalize(globals.light.xyz);
    let diffuse = max(dot(normal, light), 0.0) + material.response.x * max(dot(-normal, light), 0.0);
    let visible = visibility(in.world);
    let base = in.color * tint;
    var color = base * (globals.light.w + (1.0 - globals.light.w) * diffuse * visible);
    if material.high.w > 0.0 {
        // F0=1 obtains NoL, GGX factor and Schlick weight from the common kernel.
        let terms = dielectric(normal, light, in.world, material.low.w, 1.0);
        var fd = 0.0;
        if material.response.w > 0.0 {
            fd = material.response.w + (1.0-material.response.w) * terms.z;
        }
        let conductor = clamp(base, vec3<f32>(0.0), vec3<f32>(1.0));
        let fm = conductor + (vec3<f32>(1.0)-conductor) * terms.z;
        let dielectric_direct = base * ((1.0-fd) * terms.x) + vec3<f32>(fd * terms.y);
        let metal_direct = fm * terms.y;
        color = base * globals.light.w + (1.0-globals.light.w) * visible
            * mix(dielectric_direct, metal_direct, material.high.w);
    } else if material.response.w > 0.0 {
        let response = dielectric(normal, light, in.world, material.low.w, material.response.w);
        color += (1.0 - globals.light.w) * visible * (vec3<f32>(response.y) - base * response.x);
    }
    if globals.environment.z != 1.0 {
        let ambient_color = base * globals.light.w;
        color = ambient_color + (color-ambient_color) * globals.environment.z;
    }
    if globals.environment.x > 0.0 {
        color += globals.environment.x * environment_light(base,normal,in.world,material);
    }
    // World-space metre grid on the floor, generated by the renderer.
    let coord = in.world.xz;
    let grid = abs(fract(coord - 0.5) - 0.5) / max(fwidth(coord), vec2<f32>(0.0001));
    let line = 1.0 - min(min(grid.x, grid.y), 1.0);
    if globals.environment.w < 0.5 && in.world.y < 0.02 && in.world.y > -0.02 && in.normal.y > 0.9 {
        color = mix(color, color + vec3<f32>(0.055, 0.075, 0.08), line * 0.65);
    }
    if globals.atmosphere.w > 0.0 || globals.atmosphere_sky.w > 0.0 {
        color = atmosphere_surface(color, in.world);
    }
    return vec4<f32>(color, 1.0);
}

// Diagnostic guides keep stable colors independent of gameplay lighting.
@fragment fn fs_guide(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

// Bounded artistic daylight model. Shared by surfaces and the background;
// no spectral scattering, clouds, solar disc or volumetric shadowing implied.
fn atmosphere_color(direction: vec3<f32>) -> vec3<f32> {
    let alignment = max(dot(direction, -normalize(globals.light.xyz)), 0.0);
    let a2 = alignment * alignment;
    let a4 = a2 * a2;
    let weight = clamp(a4 * a4 * globals.atmosphere_sun.w * globals.environment.z, 0.0, 1.0);
    return mix(globals.atmosphere.xyz, globals.atmosphere_sun.xyz, weight);
}
struct AtmosphereOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) ray: vec3<f32>,
};
@vertex fn vs_atmosphere(@builtin(vertex_index) index: u32) -> AtmosphereOut {
    let points = array<vec2<f32>, 3>(vec2<f32>(-1.0,-1.0), vec2<f32>(3.0,-1.0), vec2<f32>(-1.0,3.0));
    let clip = vec4<f32>(points[index], 1.0, 1.0);
    var out: AtmosphereOut;
    out.clip = clip;
    out.ray = globals.atmosphere_forward.xyz + clip.x * globals.atmosphere_right.xyz + clip.y * globals.atmosphere_up.xyz;
    return out;
}
@fragment fn fs_atmosphere(in: AtmosphereOut) -> @location(0) vec4<f32> {
    let direction = normalize(in.ray);
    if globals.atmosphere_sky.w > 0.0 {
        let a = max(dot(direction, normalize(-globals.light.xyz)), 0.0);
        let weight = clamp(pow(a, 8.0) * globals.atmosphere_sun.w * globals.environment.z, 0.0, 1.0);
        return vec4<f32>(mix(globals.atmosphere_sky.xyz, globals.atmosphere_sun.xyz, weight), 1.0);
    }
    return vec4<f32>(atmosphere_color(direction), 1.0);
}

struct DistantOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) @interpolate(flat) color: vec3<f32>,
};
@vertex fn vs_distant(@location(0) position: vec3<f32>, @location(1) color: vec3<f32>) -> DistantOut {
    var out:DistantOut;
    out.clip=globals.view_projection*vec4<f32>(position,1.0);
    out.world=position; out.color=color; return out;
}
// No procedural noise, material table, normals, shadow map or environment lookup.
@fragment fn fs_distant(in:DistantOut) -> @location(0) vec4<f32> {
    return vec4<f32>(atmosphere_surface(in.color,in.world),1.0);
}
fn atmosphere_surface(surface: vec3<f32>, world: vec3<f32>) -> vec3<f32> {
    let ray = world - globals.view_position.xyz;
    let distance = length(ray);
    var color = surface;
    if globals.atmosphere_sky.w > 0.0 {
        let fade = smoothstep(globals.atmosphere_detail.x, globals.atmosphere_detail.y, distance) * globals.atmosphere_base.w;
        color = mix(color, globals.atmosphere_base.xyz, fade);
    }
    let transmission = exp(-globals.atmosphere.w * distance);
    return mix(atmosphere_color(ray / max(distance,0.000001)),color,transmission);
}

struct StaticInstance {
    @location(6) x: vec4<f32>, @location(7) y: vec4<f32>,
    @location(8) z: vec4<f32>, @location(9) translation: vec4<f32>,
    @location(10) tint: vec3<f32>, @location(11) material: u32,
};
fn shared_world(i: StaticInstance, p: vec3<f32>) -> vec3<f32> {
    return (mat4x4<f32>(i.x, i.y, i.z, i.translation) * vec4<f32>(p,1.0)).xyz;
}
@vertex fn vs_shared(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) color: vec3<f32>,
    @location(3) rest: vec3<f32>, @location(4) material: u32, @location(5) weights: vec3<f32>, i: StaticInstance) -> VertexOut {
    var out: VertexOut;
    out.world = shared_world(i,position);
    out.clip = globals.view_projection * vec4<f32>(out.world,1.0);
    out.normal = normalize(mat3x3<f32>(i.x.xyz,i.y.xyz,i.z.xyz) * normal);
    out.color = color*i.tint.xyz; out.rest = rest; out.material = material; out.material_projection_weights = weights;
    return out;
}
@vertex fn vs_shared_shadow(@location(0) position: vec3<f32>, i: StaticInstance) -> @builtin(position) vec4<f32> {
    return globals.shadow_projection * vec4<f32>(shared_world(i,position),1.0);
}
// Static shared vertices have white tint and one material per asset. Keep those
// values once per instance instead of repeating16 bytes at every vertex.
@vertex fn vs_shared_compact(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>,
    @location(3) rest: vec3<f32>, @location(5) weights: vec3<f32>, i: StaticInstance) -> VertexOut {
    var out: VertexOut;
    out.world = shared_world(i,position);
    out.clip = globals.view_projection * vec4<f32>(out.world,1.0);
    out.normal = normalize(mat3x3<f32>(i.x.xyz,i.y.xyz,i.z.xyz) * normal);
    out.color = i.tint; out.rest = rest; out.material = i.material; out.material_projection_weights = weights;
    return out;
}
@vertex fn vs_shared_distant(@location(0) position: vec3<f32>, i: StaticInstance) -> DistantOut {
    var out: DistantOut;
    out.world = shared_world(i,position); out.clip = globals.view_projection*vec4<f32>(out.world,1.0);
    out.color = i.tint.xyz; return out;
}

// Keep the main clip coordinates (including its near/far clipping). Only the
// depth stored by the auxiliary pass is reversed; this cannot add near-plane
// coverage that the actual surface does not have.
struct OcclusionOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) eye_depth: f32,
};
@vertex fn vs_shared_occlusion(@location(0) position: vec3<f32>, i: StaticInstance) -> OcclusionOut {
    var out: OcclusionOut;
    out.clip = globals.view_projection * vec4<f32>(shared_world(i,position),1.0);
    out.eye_depth = out.clip.w;
    return out;
}
@fragment fn fs_occlusion(in: OcclusionOut) -> @builtin(frag_depth) f32 {
    return 0.05001 / in.eye_depth;
}

@vertex fn vs_occlusion(@location(0) position: vec3<f32>) -> OcclusionOut {
    var out: OcclusionOut;
    out.clip = globals.view_projection * vec4<f32>(position,1.0);
    out.eye_depth = out.clip.w;
    return out;
}

// Material eligibility is validated before upload. Retain full-precision position
// and normal; no procedural coordinate or projection weight is consumed here.
@vertex fn vs_shared_minimal(@location(0) position: vec3<f32>, @location(1) normal:vec3<f32>, i:StaticInstance) -> VertexOut {
    var out:VertexOut;
    out.world=shared_world(i,position);
    out.clip=globals.view_projection * vec4<f32>(out.world,1.0);
    out.normal=normalize(mat3x3<f32>(i.x.xyz,i.y.xyz,i.z.xyz)*normal);
    out.color=i.tint; out.material=i.material;
    out.rest=vec3<f32>(0.0); out.material_projection_weights=vec3<f32>(0.0);
    return out;
}
