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
    planet_optics: vec4<f32>,
    planet_radial: vec4<f32>,
    planet_space: vec4<f32>,
    planet_eye_high:vec4<f32>,planet_eye_low:vec4<f32>,
    planet_axis0:vec4<f32>,planet_axis1:vec4<f32>,planet_axis2:vec4<f32>,
    direct_light:vec4<f32>,
};
@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var shadow_map: texture_depth_2d;
@group(0) @binding(2) var shadow_sampler: sampler_comparison;
fn world_ambient_visibility(world:vec3<f32>,normal:vec3<f32>)->f32 {return 1.;}

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
    surface: vec4<f32>,
    emissive: vec4<f32>,
    glass: vec4<f32>,
    attenuation: vec4<f32>,
    coverage: SurfacePattern,
    procedural: vec4<u32>,
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
    @location(6) chart0: vec4<f32>,
    @location(7) chart1: vec4<f32>,
    @location(8) chart2: vec4<f32>,
    @location(9) chart_weights: vec4<f32>,
    @location(10) chart_last: vec4<f32>,
    @location(11) chart_evolution: vec4<f32>,
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
struct ShadowOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) rest: vec3<f32>,
    @location(1) @interpolate(flat) material: u32,
};
@vertex fn vs_shadow(@location(0) position: vec3<f32>, @location(3) rest: vec3<f32>, @location(4) material: u32) -> ShadowOut {
    var out: ShadowOut;
    out.clip = globals.shadow_projection * vec4<f32>(position, 1.0);
    out.rest = rest; out.material = material;
    return out;
}
// Compile-time diagnostic oracle; other shader consumers retain the legacy default.
override cull_zero_light_shadow: bool = false;
const quality_shadow_filter: bool = false;
fn visibility(world: vec3<f32>) -> f32 {
    if globals.shadow_settings.x < 0.5 { return 1.0; }
    let p = globals.shadow_projection * vec4<f32>(world, 1.0);
    let ndc = p.xyz / p.w;
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    var gradient = vec2<f32>(0.0);
    var footprint_bias = 0.0;
    if globals.shadow_settings.w > 0.5 {
        // Differentiate geometry, not the bumped shading normal. Compute before
        // the per-fragment light-volume branch, including quads at its boundary.
        let dx = dpdx(vec3<f32>(uv, ndc.z));
        let dy = dpdy(vec3<f32>(uv, ndc.z));
        let determinant = dx.x*dy.y - dx.y*dy.x;
        let scale = length(dx.xy)*length(dy.xy);
        if abs(determinant) > max(1e-20, scale*1e-4) {
            let slope = vec2<f32>(dx.z*dy.y-dy.z*dx.y, dx.x*dy.z-dy.x*dx.z) / determinant;
            // Bound grazing amplification to 1% of light depth per texel. An
            // ill-conditioned receiver uses the legacy constant bias instead.
            let limit = 0.01 / globals.shadow_settings.z;
            gradient = clamp(slope, vec2<f32>(-limit), vec2<f32>(limit));
            // Hardware comparison filtering compares one reference with the
            // four neighbouring depths: account for the remaining footprint.
            footprint_bias = dot(abs(gradient), vec2<f32>(globals.shadow_settings.z));
        }
    }
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || ndc.z < 0.0 || ndc.z > 1.0 {
        return 1.0;
    }
    var sum = 0.0;
    var weights = 0.0;
    let radius = 1;
    for (var y = -radius; y <= radius; y++) {
        for (var x = -radius; x <= radius; x++) {
            let offset = vec2<f32>(f32(x), f32(y)) * globals.shadow_settings.z * select(1.0, 1.5, quality_shadow_filter);
            var reference = ndc.z - globals.shadow_settings.y;
            if globals.shadow_settings.w > 0.5 {
                reference += dot(gradient, offset) - footprint_bias;
            }
            let weight = select(1.0, f32((2-abs(x))*(2-abs(y))), quality_shadow_filter);
            sum += weight * textureSampleCompareLevel(shadow_map, shadow_sampler, uv + offset, reference);
            weights += weight;
        }
    }
    return sum / weights;
}
fn selected_visibility(world: vec3<f32>, needed: bool) -> f32 {
    if globals.shadow_settings.x < 0.5 { return 1.0; }
    let p = globals.shadow_projection * vec4<f32>(world, 1.0);
    let ndc = p.xyz / p.w;
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    var gradient = vec2<f32>(0.0);
    var footprint_bias = 0.0;
    if globals.shadow_settings.w > 0.5 {
        // Differentiate geometry, not the bumped shading normal. Compute before
        // the per-fragment light-volume branch, including quads at its boundary.
        let dx = dpdx(vec3<f32>(uv, ndc.z));
        let dy = dpdy(vec3<f32>(uv, ndc.z));
        let determinant = dx.x*dy.y - dx.y*dy.x;
        let scale = length(dx.xy)*length(dy.xy);
        if abs(determinant) > max(1e-20, scale*1e-4) {
            let slope = vec2<f32>(dx.z*dy.y-dy.z*dx.y, dx.x*dy.z-dy.x*dx.z) / determinant;
            // Bound grazing amplification to 1% of light depth per texel. An
            // ill-conditioned receiver uses the legacy constant bias instead.
            let limit = 0.01 / globals.shadow_settings.z;
            gradient = clamp(slope, vec2<f32>(-limit), vec2<f32>(limit));
            // Hardware comparison filtering compares one reference with the
            // four neighbouring depths: account for the remaining footprint.
            footprint_bias = dot(abs(gradient), vec2<f32>(globals.shadow_settings.z));
        }
    }
    // Derivatives above remain uniform even when direct contribution is zero.
    if !needed { return 1.0; }
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || ndc.z < 0.0 || ndc.z > 1.0 {
        return 1.0;
    }
    var sum = 0.0;
    var weights = 0.0;
    let radius = 1;
    for (var y = -radius; y <= radius; y++) {
        for (var x = -radius; x <= radius; x++) {
            let offset = vec2<f32>(f32(x), f32(y)) * globals.shadow_settings.z * select(1.0, 1.5, quality_shadow_filter);
            var reference = ndc.z - globals.shadow_settings.y;
            if globals.shadow_settings.w > 0.5 {
                reference += dot(gradient, offset) - footprint_bias;
            }
            let weight = select(1.0, f32((2-abs(x))*(2-abs(y))), quality_shadow_filter);
            sum += weight * textureSampleCompareLevel(shadow_map, shadow_sampler, uv + offset, reference);
            weights += weight;
        }
    }
    return sum / weights;
}
fn hash_cell(p: vec3<i32>, seed: u32) -> f32 {
    let q = bitcast<vec3<u32>>(p);
    var h = (q.x * 1597334677u) ^ (q.y * 3812015801u) ^ (q.z * 2798796415u) ^ seed;
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
struct EnvironmentResponse {diffuse:vec3<f32>,specular:vec3<f32>};
fn environment_response(base: vec3<f32>, normal: vec3<f32>, world: vec3<f32>, material: SurfaceMaterial) -> EnvironmentResponse {
    let metallic = material.high.w;
    let f0 = material.response.w;
    var result = vec3<f32>(0.0);
    var specular = vec3<f32>(0.0);
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
                specular += radiance * mix(dielectric_color,metal_color,metallic);
            }
        }
    }
    return EnvironmentResponse(result,specular);
}
fn environment_light(base:vec3<f32>,normal:vec3<f32>,world:vec3<f32>,material:SurfaceMaterial)->vec3<f32> {let r=environment_response(base,normal,world,material);return r.diffuse+r.specular;}
// Known at pipeline compilation; enabled only when the asset has no procedural channels.
override uniform_material: bool = false;
fn glass_transmission(base: vec3<f32>, normal: vec3<f32>, world: vec3<f32>, material: SurfaceMaterial) -> vec3<f32> {
    let to_view = globals.view_position.xyz-world;
    let view = to_view / max(length(to_view), 0.000001);
    let no_v = clamp(abs(dot(normal,view)), 0.001, 1.0);
    let fresnel = select(0.0, material.response.w + (1.0-material.response.w)*pow(1.0-no_v,5.0), material.response.w > 0.0);
    let cos_inside = sqrt(max(1.0-(1.0-no_v*no_v)/(material.glass.x*material.glass.x), 0.000001));
    let path = material.glass.y / (material.glass.z * cos_inside);
    if material.glass.y == 0.0 { return vec3<f32>(1.0-fresnel); }
    let tint = clamp(base * material.attenuation.xyz, vec3<f32>(0.0), vec3<f32>(1.0));
    // Preserve exact zero absorption channels without evaluating pow(0,0).
    let absorption = select(vec3<f32>(0.0), pow(max(tint,vec3<f32>(1e-30)), vec3<f32>(path)), tint > vec3<f32>(0.0));
    return absorption * (1.0-fresnel);
}
// Recipe1. Packed immutable programs and fixed, box-filtered periodic mip maps.
struct ProceduralData { items: array<vec4<f32>>, };
@group(0) @binding(8) var<storage,read> procedural_data: ProceduralData;
struct SurfaceSample { color:vec4<f32>, response:vec4<f32>, emission:vec4<f32>, };
fn sample_mix(a:SurfaceSample,b:SurfaceSample,t:f32)->SurfaceSample {
    return SurfaceSample(mix(a.color,b.color,t),mix(a.response,b.response,t),mix(a.emission,b.emission,t));
}
fn sample_load(at:u32)->SurfaceSample {return SurfaceSample(procedural_data.items[at],procedural_data.items[at+1u],procedural_data.items[at+2u]);}
fn periodic_hash(p:vec2<i32>,n:vec2<i32>,seed:u32)->f32 {
    let w=((p%n)+n)%n;let q=vec2<u32>(w);
    var h=(q.x*1597334677u) ^ (q.y*3812015801u) ^ seed;
    h=(h^(h>>16u))*2246822519u;h=(h^(h>>13u))*3266489917u;h=h^(h>>16u);
    return f32(h&16777215u)/16777215.0;
}
fn periodic_noise(uv:vec2<f32>,n:vec2<i32>,seed:u32)->f32 {
    let p=uv*vec2<f32>(n);let b=vec2<i32>(floor(p));let f=fract(p);let w=f*f*(3.0-2.0*f);
    return mix(mix(periodic_hash(b,n,seed),periodic_hash(b+vec2<i32>(1,0),n,seed),w.x),
        mix(periodic_hash(b+vec2<i32>(0,1),n,seed),periodic_hash(b+vec2<i32>(1,1),n,seed),w.x),w.y);
}
fn moss_fibres(uv:vec2<f32>,n:vec2<i32>,seed:u32,aa:f32)->f32 {
    let p=uv*vec2<f32>(n);let base=vec2<i32>(floor(p));var value=0.0;
    for(var y=-1;y<=1;y++){for(var x=-1;x<=1;x++){
        let c=base+vec2<i32>(x,y);let d=p-vec2<f32>(c)-vec2<f32>(periodic_hash(c,n,seed+11u),periodic_hash(c,n,seed+23u));
        let radius=length(d);if radius>=0.8 {continue;}
        var tuft=0.0;
        let sprout_count=2u+u32(periodic_hash(c,n,seed+101u)*2.0);
        for(var sprout=0u;sprout<sprout_count;sprout++){
            let channel=137u+sprout*293u;let phase=periodic_hash(c,n,seed+channel)*6.28318530718;
            let si=sin(phase);let co=cos(phase);let q=vec2<f32>(d.x*co+d.y*si,-d.x*si+d.y*co);
            let extent=0.30+0.30*periodic_hash(c,n,seed+channel+1u);
            if q.x < -0.04 || q.x>extent+0.2 {continue;}
            let curl=(periodic_hash(c,n,seed+channel+2u)-0.5)*0.28;
            let axis=curl*(q.x/extent)*(q.x/extent);
            var shoot=(1.0-smoothstep(0.004-aa,0.012+aa,abs(q.y-axis)))*smoothstep(0.0,0.04,q.x)*(1.0-smoothstep(extent*0.85,extent,q.x))*0.45;
            let leaf_count=3u+u32(periodic_hash(c,n,seed+channel+19u)*3.0);
            for(var k=0u;k<leaf_count;k++){for(var side_index=0u;side_index<2u;side_index++){
                let leaf=channel+11u+k*17u+side_index*83u;let side=select(-1.0,1.0,side_index==1u);
                let t=extent*(f32(k)+0.35+0.55*periodic_hash(c,n,seed+leaf))/f32(leaf_count);
                let root=vec2<f32>(t,curl*(t/extent)*(t/extent));
                let dir=normalize(vec2<f32>(0.35+0.6*periodic_hash(c,n,seed+leaf+1u),side*(0.65+0.3*periodic_hash(c,n,seed+leaf+2u))));
                let across=vec2<f32>(-dir.y,dir.x);let v=q-root;
                let leaf_length=(0.08+0.10*periodic_hash(c,n,seed+leaf+3u))*(1.0-0.25*t/extent);
                let along=dot(v,dir)/leaf_length;if along<0.0 || along>1.0 {continue;}
                let width=(0.014+0.020*periodic_hash(c,n,seed+leaf+4u))*sqrt(max(0.0,sin(3.14159265359*along)));
                let blade=(1.0-smoothstep(width*0.15-aa,width+aa+0.003,abs(dot(v,across))))*smoothstep(0.0,0.12,along)*(1.0-smoothstep(0.75,1.0,along));
                shoot=max(shoot,blade*(0.5+0.5*periodic_hash(c,n,seed+leaf+5u)));
            }}tuft=max(tuft,shoot*(0.65+0.35*periodic_hash(c,n,seed+channel+7u)));
        }value=max(value,tuft*(1.0-smoothstep(0.65,0.8,radius)));
    }}return clamp(value,0.0,1.0);
}
fn recipe_field(uv:vec2<f32>,n:vec2<i32>,seed:u32,kind:f32,aa:f32)->f32 {
    if kind>4.5 {return moss_fibres(uv,n,seed,aa);}
    if kind<0.5 {return periodic_noise(uv,n,seed);}
    if kind<1.5 {return 1.0-abs(2.0*periodic_noise(uv,n,seed)-1.0);}
    if kind>3.5 {return 0.5+0.5*sin(6.28318530718*(uv.x*f32(n.x)+periodic_noise(uv,vec2<i32>(2),seed)));}
    var p=uv*vec2<f32>(n);
    if kind>2.5 {
        p.x+=0.5*f32(((i32(floor(p.y))%2)+2)%2)*select(0.0,1.0,n.y%2==0);
        let b=vec2<i32>(floor(p));let f=fract(p);let edge=min(min(f.x,1.0-f.x),min(f.y,1.0-f.y));
        return smoothstep(0.035-aa,0.065+aa,edge)*(0.7+0.3*periodic_hash(b,n,seed));
    }
    let b=vec2<i32>(floor(p));var first=100.0;var second=100.0;
    for(var y=-1;y<=1;y++){for(var x=-1;x<=1;x++){
        let c=b+vec2<i32>(x,y);
        let delta=p-vec2<f32>(c)-vec2<f32>(periodic_hash(c,n,seed),periodic_hash(c,n,seed+11u));
        let d=length(delta);if d<first {second=first;first=d;}else if d<second {second=d;}
    }}
    return smoothstep(0.02-aa,0.12+aa,second-first);
}
fn fixed_texel(at:u32,size:u32,p:vec2<i32>)->SurfaceSample {
    let n=i32(size);let q=((p%n)+n)%n;
    return sample_load(at+3u*(u32(q.y)*size+u32(q.x)));
}
fn fixed_gpu_sample(info:vec4<u32>,uv:vec2<f32>,lod:f32)->SurfaceSample { return sample_load(info.y+2u); }
fn fixed_level(info:vec4<u32>,uv:vec2<f32>,level:u32)->SurfaceSample {
    var size=info.z;var at=info.y+6u;
    for(var l=0u;l<level;l++){at+=3u*size*size;size/=2u;}
    let p=fract(uv)*f32(size)-0.5;let b=vec2<i32>(floor(p));let f=fract(p);
    return sample_mix(sample_mix(fixed_texel(at,size,b),fixed_texel(at,size,b+vec2<i32>(1,0)),f.x),
        sample_mix(fixed_texel(at,size,b+vec2<i32>(0,1)),fixed_texel(at,size,b+vec2<i32>(1,1)),f.x),f.y);
}
// Marble1: periodic mineral domains, normalized fracture distance and fine grain.
fn marble_noise(uv:vec2<f32>,n:i32,seed:u32,fp:f32)->f32 {
    let coverage=1.0-smoothstep(0.25,0.75,fp*f32(n));
    if coverage<=0.0 {return 0.5;}
    return mix(0.5,periodic_noise(uv,vec2<i32>(n),seed),coverage);
}
fn marble_grain(uv:vec2<f32>,n:i32,seed:u32,fp:f32)->f32 {
    let coverage=1.0-smoothstep(1.0,3.0,fp*f32(n));if coverage<=0.0 {return 0.5;}
    let p=uv*f32(n);let offset=fp*f32(n)*0.25;var sum=0.0;
    for(var y=-1;y<=1;y+=2){for(var x=-1;x<=1;x+=2){sum+=periodic_hash(vec2<i32>(floor(p+vec2<f32>(f32(x),f32(y))*offset)),vec2<i32>(n),seed);}}
    return mix(0.5,sum*0.25,coverage);
}
fn marble_fbm(uv:vec2<f32>,n:i32,seed:u32,octaves:u32,fp:f32)->f32 {
    var value=0.0;var total=0.0;var weight=0.5;
    for(var i=0u;i<5u;i++) {
        if i>=octaves {break;}
        var q=uv;var footprint=fp;
        if i%2u!=0u {q=vec2<f32>(uv.x+uv.y,uv.y-uv.x);footprint*=1.4142135623730951;}
        value+=weight*marble_noise(q,n*i32(1u<<i),seed+i*1013u,footprint);total+=weight;weight*=0.5;
    }
    return value/total;
}
fn marble_cell(uv:vec2<f32>,n:i32,seed:u32)->vec2<f32> {
    let p=uv*f32(n);let base=vec2<i32>(floor(p));var first=100.0;var second=100.0;var a=vec2<f32>(0.0);var b=vec2<f32>(0.0);var domain=0.0;
    for(var y=-1;y<=1;y++){for(var x=-1;x<=1;x++) {
        let c=base+vec2<i32>(x,y);let q=vec2<f32>(c)+vec2<f32>(periodic_hash(c,vec2<i32>(n),seed),periodic_hash(c,vec2<i32>(n),seed+11u));let d=dot(p-q,p-q);
        if d<first {second=first;b=a;first=d;a=q;domain=periodic_hash(c,vec2<i32>(n),seed+912u);}else if d<second {second=d;b=q;}
    }}
    return vec2<f32>(0.5*(second-first)/max(length(b-a),0.00001),domain);
}
fn marble_sample(at:u32,uv:vec2<f32>,footprint:f32,seed:u32)->SurfaceSample {
    let dark=procedural_data.items[at];let light=procedural_data.items[at+1u];let calcite=procedural_data.items[at+2u];let gilding=procedural_data.items[at+3u];
    let structure=procedural_data.items[at+4u];let finish=procedural_data.items[at+5u];let shape=procedural_data.items[at+6u];let details=procedural_data.items[at+7u];
    let a=marble_fbm(uv,2,seed+17u,5u,footprint);let b=marble_fbm(uv,3,seed+53u,5u,footprint);
    let q=uv+structure.z*(vec2<f32>(a,b)-0.5)+0.004*(vec2<f32>(marble_noise(uv,96,seed+71u,footprint),marble_noise(uv,96,seed+89u,footprint))-0.5);
    let fp=footprint*(1.0+structure.z*3.0);
    let cloud=marble_fbm(q,4,seed+97u,5u,fp);let small=marble_fbm(q,15,seed+131u,3u,fp);
    let flow=marble_fbm(uv,2,seed+17u,3u,footprint);
    let phase=dot(uv,shape.xy)+shape.z*(flow-0.5)+0.015*(small-0.5);
    let bands=0.5+0.5*sin(6.28318530718*phase);let folded=0.5+0.5*sin(6.28318530718*(phase*3.0+small*0.7));
    var value=smoothstep(0.22,0.76,cloud+0.17*(bands-0.5));value=mix(value,smoothstep(0.1,0.9,bands)*0.65+folded*0.35,structure.w);
    let strata=0.5+0.5*sin(6.28318530718*(phase*32.0+small*2.4))*(1.0-smoothstep(0.25,0.75,fp*64.0));let patches=marble_fbm(uv,27,seed+173u,5u,footprint);
    value=pow(clamp(value+0.6*(patches-0.5)+structure.w*0.22*(strata-0.5),0.0,1.0),light.w);
    var color=mix(dark.xyz,light.xyz,value);
    let primary=marble_cell(q,i32(structure.x),seed+211u);let secondary=marble_cell(q,10,seed+307u);let hairline=marble_cell(q,47,seed+401u);
    let aa=min(fp*structure.x*0.5,0.15);let width_variation=smoothstep(0.2,0.8,small);let width=structure.y*(0.15+2.8*width_variation*width_variation);
    let network_width=width*mix(1.0,0.12,finish.x);
    let network=1.0-smoothstep(max(0.0,network_width*0.85-aa),network_width*1.05+aa,primary.x);
    let main_width=structure.y*(0.65+0.7*small);let line=abs(sin(3.14159265359*phase));let main=1.0-smoothstep(max(0.0,main_width*0.9-footprint),main_width*1.05+footprint,line);
    let fine=(1.0-smoothstep(0.003-fp*5.0,0.013+fp*5.0,secondary.x))*smoothstep(0.32,0.72,cloud+0.25*small);
    let hair=(1.0-smoothstep(0.001-fp*23.5,0.006+fp*23.5,hairline.x))*smoothstep(0.42,0.7,small)*0.65;
    let fine_fade=1.0-smoothstep(0.001,0.012,fp);
    let white=max(max(network*mix(1.0,0.65,finish.x),main*finish.x),max(fine,hair)*finish.y*fine_fade)*calcite.w;
    color*=(1.0+finish.z*2.0*(patches-0.5))*mix(1.0,0.35+(secondary.y*0.35+hairline.y*0.65)*1.2,details.x);
    var quartz=mix(calcite.xyz*0.45+light.xyz*0.1,calcite.xyz,smoothstep(0.15,0.85,marble_fbm(uv,40,seed+487u,3u,footprint)));
    quartz=mix(quartz,calcite.xyz,main*0.65);
    color=mix(color,light.xyz,details.x*smoothstep(0.77,0.94,hairline.y)*0.45);
    color=mix(color,quartz,white);
    let gw=shape.w*(0.15+2.8*width_variation*width_variation);
    let gold_network_width=gw*mix(1.0,0.12,details.z);
    var gold=(1.0-smoothstep(max(0.0,gold_network_width*0.85-aa),gold_network_width*1.05+aa,primary.x))*gilding.w;
    let flow_width=shape.w*(0.65+0.7*small);
    let flow_gold=1.0-smoothstep(max(0.0,flow_width*0.85-footprint),flow_width*1.05+footprint,line);
    gold=max(gold,flow_gold*details.z*gilding.w);
    gold=max(gold,(1.0-smoothstep(0.002-fp*5.0,0.009+fp*5.0,secondary.x))*smoothstep(0.34,0.8,cloud)*gilding.w*0.8*fine_fade);
    if finish.x>0.0 {
        let halo=(1.0-smoothstep(main_width*1.05,main_width*1.25+footprint,line))*(1.0-main)*smoothstep(0.38,0.62,small);
        gold=max(gold,halo*gilding.w);
    }
    let grain=marble_grain(uv,i32(details.y),seed+503u,footprint);
    let gold_color=mix(gilding.xyz*0.45,gilding.xyz,smoothstep(0.15,0.85,grain));
    color=mix(color,gold_color,gold);
    let flecks=smoothstep(0.6,0.94,grain)*finish.z*0.22*(1.0-white)*(1.0-gold);
    color=mix(color,mix(light.xyz,calcite.xyz,0.25),flecks);
    color=clamp(color,vec3<f32>(0.0),vec3<f32>(2.0));
    let roughness=clamp(mix(dark.w+0.025*(grain-0.5),0.27+0.05*grain,gold),0.2,1.0);
    return SurfaceSample(vec4<f32>(color,roughness),vec4<f32>(gold*finish.w,0.5+0.015*(grain-0.5)+0.035*white+0.015*gold,1.0,1.0),vec4<f32>(0.0));
}

// Rock1 shares deterministic periodic noise with the mineral programs.
fn rock_cell(uv:vec2<f32>,n:vec2<i32>,seed:u32)->vec4<f32> {
    let p=uv*vec2<f32>(n);let base=vec2<i32>(floor(p));var first=100.0;var second=100.0;var a=vec2<f32>(0.0);var b=vec2<f32>(0.0);var id=0.0;
    for(var y=-1;y<=1;y++){for(var x=-1;x<=1;x++){
        let c=base+vec2<i32>(x,y);let q=vec2<f32>(c)+0.15+0.7*vec2<f32>(periodic_hash(c,n,seed),periodic_hash(c,n,seed+11u));let d=dot(p-q,p-q);
        if d<first {second=first;b=a;first=d;a=q;id=periodic_hash(c,n,seed+912u);}else if d<second {second=d;b=q;}
    }}
    return vec4<f32>(0.5*(second-first)/max(length(b-a),0.00001),id,p-a);
}
fn rock_sample(at:u32,uv:vec2<f32>,footprint:f32,seed:u32)->SurfaceSample {
    let dark=procedural_data.items[at];let light=procedural_data.items[at+1u];let oxide=procedural_data.items[at+2u];let mineral=procedural_data.items[at+3u];
    let blocks=procedural_data.items[at+4u];let finish=procedural_data.items[at+5u];let strata=procedural_data.items[at+6u];let shape=procedural_data.items[at+7u];let detail=procedural_data.items[at+8u];
    let a=marble_fbm(uv,2,seed+31u,4u,footprint);let b=marble_fbm(uv,3,seed+71u,4u,footprint);
    let q=vec2<f32>(uv.x+shape.x*uv.y,uv.y)+finish.x*(vec2<f32>(a,b)-0.5)+0.006*(vec2<f32>(marble_noise(uv,73,seed+89u,footprint),marble_noise(uv,73,seed+101u,footprint))-0.5);
    let fp=footprint*(1.0+finish.x*3.0+abs(shape.x));let n=vec2<i32>(blocks.xy);
    let primary=rock_cell(q,n,seed+151u);let secondary=rock_cell(q,n*vec2<i32>(3,2),seed+233u);
    let coarse=marble_fbm(q,7,seed+317u,4u,fp);let small=marble_fbm(q,27,seed+389u,4u,fp);
    let grain=marble_grain(uv,i32(finish.w),seed+431u,footprint);
    let aa=min(fp*f32(max(n.x,n.y))*0.5,0.18);let width=blocks.z*(0.25+2.5*pow(smoothstep(0.2,0.8,small),2.0));
    let main=1.0-smoothstep(max(0.0,width*0.45-aa),width+aa,primary.x);
    let sub=(1.0-smoothstep(max(0.0,width*0.12-aa*2.0),width*0.4+aa*2.0,secondary.x))*smoothstep(0.27,0.7,coarse)*detail.z;
    let joint=max(main,sub);let slope=fract(primary.y*vec2<f32>(17.0,43.0))-0.5;
    let plane=0.62+0.3*(primary.y-0.5)+1.5*dot(primary.zw,slope);
    let cut=0.9-0.75*abs(primary.z*0.7-primary.w*0.8+primary.y-0.5);
    let bevel=smoothstep(width,width+0.12*shape.w+0.005,primary.x);
    let face=mix(0.6,clamp(min(plane,cut),0.15,0.95),finish.y)*mix(1.0,0.7+0.3*bevel,shape.w);
    let count=i32(strata.x);var phase=dot(uv,shape.xy)*strata.x+strata.z*strata.x*0.2*(a-0.5)+0.12*(small-0.5);
    phase+=strata.w*1.6*(periodic_noise(vec2<f32>(phase/strata.x,0.37),vec2<i32>(max(count/3,1),1),seed+503u)-0.5);
    let layer=i32(floor(phase));let t=fract(phase);let hardness=periodic_hash(vec2<i32>(layer,0),vec2<i32>(count,1),seed+547u);
    let rise=smoothstep(0.015,0.08,t);let lip=1.0-smoothstep(0.68,0.98,t);let shelf=rise*lip;
    let lamination=mix(0.5,abs(sin(3.14159265359*(phase*4.0+small*0.65))),1.0-smoothstep(0.25,0.75,footprint*strata.x*4.0*length(shape.xy)));
    let layer_height=0.24+0.48*hardness+0.2*shelf-0.045*(1.0-lamination)-detail.w*0.2*(1.0-shelf);let pits=smoothstep(0.57,0.78,small)*shape.z;
    let height=clamp(mix(face,layer_height,strata.y*0.9)-joint*blocks.w*0.5-pits*0.12+detail.y*((small-0.5)*0.75+(marble_noise(uv,96,seed+467u,footprint)-0.5)*0.08),0.02,0.98);
    var value=mix(0.34+0.46*coarse+0.22*(primary.y-0.5),0.18+0.68*hardness+0.12*shelf,strata.y);
    value=pow(clamp(value+finish.z*0.24*(grain-0.5)+0.35*(small-0.5)+strata.y*0.06*(lamination-0.5),0.0,1.0),light.w);
    var color=mix(dark.xyz,light.xyz,value);let rust=mix(smoothstep(0.46,0.76,small)*(joint*0.65+smoothstep(0.68,0.83,coarse)*0.35),smoothstep(0.25,0.72,hardness)*(0.55+0.45*small),strata.y)*oxide.w;
    color=mix(color,oxide.xyz,rust);let crystals=smoothstep(0.55,0.8,small)*smoothstep(0.59,0.86,grain)*mineral.w;
    let seams=(1.0-smoothstep(0.008,0.035,t))*smoothstep(0.82,0.96,hardness)*strata.y*mineral.w;
    color=mix(color,mineral.xyz,max(crystals,seams));
    let roughness=clamp(dark.w+detail.x*(grain-0.5)+0.1*pits+0.06*joint,0.2,1.0);
    let ao=clamp(1.0-0.5*joint*blocks.w-0.2*(1.0-shelf)*strata.y-0.12*pits,0.25,1.0);
    return SurfaceSample(vec4<f32>(color,roughness),vec4<f32>(0.0,height,1.0,ao),vec4<f32>(0.0));
}
// Rock2 coats a continuous surface. Large fracture planes belong to source geometry.
fn rock_surface_sample(at:u32,uv:vec2<f32>,footprint:f32,seed:u32)->SurfaceSample {
    let dark=procedural_data.items[at];let light=procedural_data.items[at+1u];let oxide=procedural_data.items[at+2u];let mineral=procedural_data.items[at+3u];
    let blocks=procedural_data.items[at+4u];let finish=procedural_data.items[at+5u];let strata=procedural_data.items[at+6u];let shape=procedural_data.items[at+7u];let detail=procedural_data.items[at+8u];
    let a=marble_fbm(uv,2,seed+31u,4u,footprint);let b=marble_fbm(uv,3,seed+71u,4u,footprint);
    let q=vec2<f32>(uv.x+shape.x*uv.y,uv.y)+finish.x*(vec2<f32>(a,b)-0.5);
    let fp=footprint*(1.0+finish.x*3.0+abs(shape.x));
    let coarse=marble_fbm(q,4,seed+317u,4u,fp);let small=marble_fbm(q,37,seed+389u,4u,fp);let fine=marble_fbm(q,128,seed+467u,2u,fp);
    let grain=marble_grain(uv,i32(finish.w),seed+431u,footprint);
    let frequency=i32(blocks.x)*7+i32(blocks.y)*3;
    let seam=abs(marble_fbm(q,frequency,seed+151u,3u,fp)-0.5);let cross=abs(marble_fbm(q,frequency*2,seed+233u,2u,fp)-0.5);
    let aa=min(fp*f32(frequency)*0.12,0.08);let width=blocks.z*(0.35+1.3*small);
    let interrupted=smoothstep(0.53,0.7,coarse)*smoothstep(0.42,0.64,small);
    let fissure=min(1.0,(1.0-smoothstep(width,width+0.008+aa,seam))+detail.z*(1.0-smoothstep(width*0.4,width*0.4+0.008+aa,cross)))*interrupted*(1.0-smoothstep(0.3,0.9,fp*f32(frequency)));
    let phase=dot(uv,shape.xy)*strata.x+strata.z*strata.x*0.15*(a-0.5)+strata.w*0.8*(small-0.5);
    let band=sin(6.28318530718*phase)*(1.0-smoothstep(0.2,0.75,footprint*strata.x*length(shape.xy)));
    let pits=smoothstep(0.62,0.83,fine)*shape.z;
    let height=clamp(0.5+detail.y*((small-0.5)*0.55+(fine-0.5)*0.3)+finish.y*0.04*(coarse-0.5)+strata.y*band*(0.015+0.025*detail.w)-fissure*blocks.w*0.2-pits*0.08,0.02,0.98);
    let value=pow(clamp(0.52+0.65*(coarse-0.5)+0.25*(small-0.5)+finish.z*0.32*(grain-0.5)+strata.y*0.17*band,0.0,1.0),light.w);
    var color=mix(dark.xyz,light.xyz,value);
    let rust=oxide.w*smoothstep(0.45,0.75,b)*(0.35+0.65*small);color=mix(color,oxide.xyz,rust);
    let crystals=smoothstep(0.54,0.76,small)*smoothstep(0.56,0.8,grain)*mineral.w;
    color=mix(color,mineral.xyz,crystals*0.65);color=mix(color,dark.xyz,fissure*blocks.w*0.25);
    let roughness=clamp(dark.w+detail.x*(grain-0.5)+shape.w*0.06*(small-0.5)+pits*0.08,0.2,1.0);
    let ao=clamp(1.0-0.12*fissure*blocks.w-0.08*pits,0.85,1.0);
    return SurfaceSample(vec4<f32>(color,roughness),vec4<f32>(0.0,height,1.0,ao),vec4<f32>(0.0));
}
override fixed_recipe: bool = false;
override anatomical_material: bool = true;
fn recipe_tile_sample(info:vec4<u32>,uv:vec2<f32>,footprint:f32)->SurfaceSample {
    if info.x==3u {
        let lod=clamp(log2(max(1.0,footprint*f32(info.z))),0.0,log2(f32(info.z)));
        return fixed_gpu_sample(info,uv,lod);
    }
    if info.x==2u {
        let last=u32(log2(f32(info.z)));let lod=clamp(log2(max(1.0,footprint*f32(info.z))),0.0,f32(last));
        let level=u32(floor(lod));
        return sample_mix(fixed_level(info,uv,level),fixed_level(info,uv,min(level+1u,last)),fract(lod));
    }
    if fixed_recipe { return sample_load(info.y+2u); }
    var sample=sample_load(info.y+2u);let metadata=procedural_data.items[info.y+5u];let uv0=fract(uv);
    let seed0=u32(metadata.x)|(u32(metadata.z)<<16u);
    var words=0u;
    if metadata.w>2.5 {sample=rock_surface_sample(info.y+6u,uv0,footprint,seed0);words=9u;}
    else if metadata.w>1.5 {sample=rock_sample(info.y+6u,uv0,footprint,seed0);words=9u;}
    else if metadata.w>0.5 {sample=marble_sample(info.y+6u,uv0,footprint,seed0);words=8u;}
    for(var layer=0u;layer<8u;layer++) {
        if layer>=u32(metadata.y) {break;}
        let at=info.y+6u+words+6u*layer;let f=procedural_data.items[at];let settings=procedural_data.items[at+1u];let mask=procedural_data.items[at+2u];
        let seed=(u32(metadata.x)|(u32(metadata.z)<<16u))+(u32(f.w)|(u32(settings.z)<<16u));var q=uv0;
        let packed_warp=u32(settings.w);let warp_n=vec2<i32>(i32(packed_warp>>8u),i32(packed_warp&255u));
        if settings.y>0.0 {q+=settings.y*(vec2<f32>(periodic_noise(uv0,warp_n,seed+11u),periodic_noise(uv0,warp_n,seed+23u))-0.5);}
        var value=0.0;var total=0.0;var weight=0.5;var n=vec2<i32>(f.yz);
        for(var octave=0u;octave<4u;octave++) {
            if octave>=u32(settings.x) {break;}
            let fp=footprint*f32(max(n.x,n.y))*(1.0+settings.y*2.0*f32(max(warp_n.x,warp_n.y)));let coverage=1.0-smoothstep(0.25,0.75,fp);
            var v=0.5;if coverage>0.0 {v=recipe_field(q,n,seed+octave*1013u,f.x,fp*0.5);}
            value+=weight*mix(0.5,v,coverage);total+=weight;weight*=0.5;n*=2;
        }
        var t=smoothstep(mask.x,mask.y,value/total);if mask.w>0.5 {t=1.0-t;}
        sample=sample_mix(sample,sample_load(at+3u),t*mask.z);
    }
    return sample;
}
// Triangular lattice blending: each shared lattice vertex owns one transform.
// A vanishing edge weight gives the same field on either side of every seam.
fn recipe_sample(info:vec4<u32>,uv:vec2<f32>,footprint:f32)->SurfaceSample {
    let flag=procedural_data.items[info.y].w;
    if flag<1.5 {return recipe_tile_sample(info,uv,footprint);}
    let options=procedural_data.items[info.y+u32(flag)-2u];let seed=u32(options.x)|(u32(options.y)<<16u);
    let skew=vec2<f32>(uv.x-uv.y*0.57735026,uv.y*1.1547005);let cell=vec2<i32>(floor(skew));let f=fract(skew);
    var offsets=array<vec2<i32>,3>(vec2<i32>(0,0),vec2<i32>(1,0),vec2<i32>(0,1));var bary=vec3<f32>(1.0-f.x-f.y,f.x,f.y);
    if f.x+f.y>1.0 {offsets=array<vec2<i32>,3>(vec2<i32>(1,1),vec2<i32>(0,1),vec2<i32>(1,0));bary=vec3<f32>(f.x+f.y-1.0,1.0-f.x,1.0-f.y);}
    var result=SurfaceSample(vec4<f32>(0.0),vec4<f32>(0.0),vec4<f32>(0.0));var total=0.0;
    for(var i=0u;i<3u;i++) {
        let c=cell+offsets[i];let site=vec3<i32>(c,0);let center=vec2<f32>(f32(c.x)+0.5*f32(c.y),f32(c.y)*0.8660254);let delta=uv-center;
        var q=delta;var turn=0u;if options.w>0.5 {turn=u32(floor(hash_cell(site,seed+31u)*4.0))%4u;}
        if turn==1u {q=vec2<f32>(-delta.y,delta.x);}else if turn==2u {q=-delta;}else if turn==3u {q=vec2<f32>(delta.y,-delta.x);}
        q+=vec2<f32>(hash_cell(site,seed+11u),hash_cell(site,seed+23u));
        let sample=recipe_tile_sample(info,q,footprint);let weight=pow(max(bary[i],0.0),options.z)*(0.25+sample.response.y*sample.response.y);
        result.color+=sample.color*weight;result.response+=sample.response*weight;result.emission+=sample.emission*weight;total+=weight;
    }
    return SurfaceSample(result.color/max(total,1e-20),result.response/max(total,1e-20),result.emission/max(total,1e-20));
}
struct AnatomyHits { ids:vec4<u32>,groups:vec4<u32>,distances:vec4<f32>,count:u32, };
fn anatomy_hits(at:u32,p:vec3<f32>)->AnatomyHits {
    let header=procedural_data.items[at];let nodes=u32(header.x);let capsules=at+5u+3u*nodes;
    let continuous=procedural_data.items[at+4u].y>0.5;let stride=select(5u,7u,continuous);
    var result=AnatomyHits(vec4<u32>(0u),vec4<u32>(0u),vec4<f32>(1e30),0u);
    var node=0u;var best=1e30;
    loop {
        if node>=nodes {break;}
        let lo=procedural_data.items[at+5u+3u*node];let hi=procedural_data.items[at+6u+3u*node];let leaf=procedural_data.items[at+7u+3u*node];
        let bound=length(max(max(lo.xyz-p,p-hi.xyz),vec3<f32>(0.0)))-hi.w;
        if bound>best+4.0*header.z {node=u32(lo.w);continue;}
        for(var i=0u;i<u32(leaf.y);i++) {
            let id=u32(leaf.x)+i;let base=capsules+stride*id;let a=procedural_data.items[base];let b=procedural_data.items[base+1u];let frame=procedural_data.items[base+2u];
            let ab=b.xyz-a.xyz;let t=clamp(dot(p-a.xyz,ab)/dot(ab,ab),0.0,1.0);let d=length(p-a.xyz-ab*t)-mix(a.w,b.w,t);best=min(best,d);
            var slot=result.count;
            for(var j=0u;j<result.count;j++) {if result.groups[j]==u32(frame.w) {slot=j;break;}}
            if slot<result.count && d>=result.distances[slot] {continue;}
            if slot==result.count && result.count==3u && d>=result.distances[2] {continue;}
            result.ids[slot]=id;result.groups[slot]=u32(frame.w);result.distances[slot]=d;
            result.count=min(3u,max(result.count,slot+1u));
            loop {
                if slot==0u {break;}
                if result.distances[slot]>=result.distances[slot-1u] {break;}
                let distance=result.distances[slot-1u];let ident=result.ids[slot-1u];let group=result.groups[slot-1u];
                result.distances[slot-1u]=result.distances[slot];result.ids[slot-1u]=result.ids[slot];result.groups[slot-1u]=result.groups[slot];
                result.distances[slot]=distance;result.ids[slot]=ident;result.groups[slot]=group;slot-=1u;
            }
        }
        node+=1u;
    }
    return result;
}
fn anatomy_grid(uv:vec2<f32>,footprint:f32)->f32 {
    let q=abs(fract(uv/0.25)-0.5);let edge=max(q.x,q.y);let aa=min(0.04,footprint/0.25);
    return mix(0.85,0.08,smoothstep(0.46-aa,0.48+aa,edge));
}
fn anatomy_rotate(q:vec4<f32>,p:vec3<f32>)->vec3<f32> {
    return p+2.0*cross(q.xyz,cross(q.xyz,p)+q.w*p);
}
// Coating is evaluated in the undeformed source rest space, outside chart blending.
// x=coverage, y=geometric offset, z=shaded height in metres, w=color variation.
fn moss_field(at:u32,p:vec3<f32>,footprint:f32)->vec4<f32> {
    let a=procedural_data.items[at];let b=procedural_data.items[at+1u];let c=procedural_data.items[at+2u];
    if a.x<=0.0 {return vec4<f32>(0.0,0.0,0.0,0.5);}
    let seed=u32(a.z)|(u32(a.w)<<16u);var mask=1.0;
    if a.x<1.0 {
        var q=p/a.y;
        if b.x>0.0 {let low=q*0.6;q+=(vec3<f32>(value_noise(low,seed+11u),value_noise(low,seed+23u),value_noise(low,seed+37u))-0.5)*(b.x*0.9);}
        let density=mix(value_noise(q,seed),value_noise(q*2.0,seed+53u),0.4*b.x);
        let score=smoothstep(0.15,0.85,density);let edge=b.y+min(0.3,footprint/a.y);
        let threshold=(1.0+edge)-(1.0+2.0*edge)*a.x;mask=smoothstep(threshold-edge,threshold+edge,score);
    }
    if mask==0.0 {return vec4<f32>(0.0,0.0,0.0,0.5);}
    let q=p/b.w;let lump=0.65*value_noise(q,seed+101u)+0.35*value_noise(q*2.0,seed+173u);
    let raw=value_noise(p/c.x,seed+307u);let grain=1.0-abs(2.0*raw-1.0);let fade=1.0-smoothstep(0.35,0.8,footprint/c.x);
    let micro=mix(0.5,grain,fade);
    var variation=clamp(0.2+0.55*micro+0.25*lump,0.0,1.0);
    if c.w>0.0 {
        let qleaf=p/c.x;let cell=vec3<i32>(floor(qleaf));
        let r=vec3<f32>(hash_cell(cell,seed+401u),hash_cell(cell,seed+409u),hash_cell(cell,seed+419u))*2.0-1.0;
        let axis=normalize(r+vec3<f32>(0.003,0.007,0.011));let reference=select(vec3<f32>(0.0,1.0,0.0),vec3<f32>(0.0,0.0,1.0),abs(axis.z)<0.8);
        let across=normalize(cross(axis,reference));let normal=cross(axis,across);
        let local=qleaf-floor(qleaf)-0.5-r*0.025;
        let leaf=vec3<f32>(dot(local,axis)/0.4,dot(local,across)/0.16,dot(local,normal)/0.27);
        let aa=min(0.1,footprint/c.x);let shape=1.0-smoothstep(0.65-aa,1.0+aa,length(leaf));
        let pigment=clamp(0.56+0.32*hash_cell(cell,seed+431u)+0.08*leaf.x,0.0,1.0);
        let background=0.18+0.15*micro+0.2*lump;
        let visible=1.0-smoothstep(0.25,0.8,footprint/c.x);
        variation=mix(variation,mix(background,pigment,shape),c.w*visible);
    }
    var height=(micro-0.5)*c.y;
    let granules=procedural_data.items[at+5u];
    if granules.x>0.0 && footprint<0.8*granules.x {
        var qgran=p/granules.x;let low=qgran*0.37;
        qgran+=(vec3<f32>(value_noise(low,seed+509u),value_noise(low,seed+521u),value_noise(low,seed+541u))-0.5)*0.65;
        let cell=vec3<i32>(floor(qgran));let local=qgran-floor(qgran)-0.5;let edge=abs(local);
        let window=(1.0-smoothstep(0.43,0.5,edge.x))*(1.0-smoothstep(0.43,0.5,edge.y))*(1.0-smoothstep(0.43,0.5,edge.z));
        let jitter=(vec3<f32>(hash_cell(cell,seed+577u),hash_cell(cell,seed+587u),hash_cell(cell,seed+599u))-0.5)*0.24;
        let radius=0.25+0.1*hash_cell(cell,seed+607u);
        let mound=1.0-smoothstep(0.08,1.0,length((local-jitter)*vec3<f32>(1.0,1.1,1.0))/radius);
        let tint=hash_cell(cell,seed+557u);let selected=select(0.0,1.0,granules.w==1.0 || hash_cell(cell,seed+569u)<granules.w);
        let visible=1.0-smoothstep(0.25,0.8,footprint/granules.x);
        let delta=granules.z*((mound-0.5)*0.5+(tint-0.5)*0.3*window)*visible;
        let dark=selected*mound*procedural_data.items[at+3u].w*visible;
        variation=clamp(variation+delta,0.0,1.0)*(1.0-dark);
        height+=(mound*(0.7+0.3*tint)-0.5)*granules.y*visible;
    }
    return vec4<f32>(mask,mask*b.z*(0.3+0.7*lump),height,variation);
}
fn anatomy_sample(info:vec4<u32>,p:vec3<f32>,dx:vec3<f32>,dy:vec3<f32>,hits:AnatomyHits)->SurfaceSample {
    let at=info.w;let header=procedural_data.items[at];let capsules=at+5u+3u*u32(header.x);let tile=procedural_data.items[info.y].xy;
    let young=procedural_data.items[at+1u];let mature=procedural_data.items[at+2u];let evolution=procedural_data.items[at+3u];let growth_response=procedural_data.items[at+4u].x;
    let continuous=procedural_data.items[at+4u].y>0.5;let stride=select(5u,7u,continuous);
    var distances=hits.distances;var nearest=hits.distances.x;var farthest=nearest;
    if continuous {
        nearest=1e30;farthest=-1e30;
        for(var i=0u;i<hits.count;i++) {
            let base=capsules+stride*hits.ids[i];let a=procedural_data.items[base];let b=procedural_data.items[base+1u];
            let ab=b.xyz-a.xyz;let t=clamp(dot(p-a.xyz,ab)/dot(ab,ab),0.0,1.0);
            let d=length(p-a.xyz-ab*t)-mix(a.w,b.w,t);distances[i]=d;nearest=min(nearest,d);farthest=max(farthest,d);
        }
    }
    var result=SurfaceSample(vec4<f32>(0.0),vec4<f32>(0.0),vec4<f32>(0.0));var total=0.0;
    for(var i=0u;i<hits.count;i++) {
        let base=capsules+stride*hits.ids[i];let a=procedural_data.items[base];let b=procedural_data.items[base+1u];let frame=procedural_data.items[base+2u];let arc=procedural_data.items[base+3u];let clocks=procedural_data.items[base+4u];
        let ab=b.xyz-a.xyz;var tangent=normalize(ab);var n=frame.xyz;var binormal=cross(tangent,n);let delta=p-a.xyz;let t=clamp(dot(delta,ab)/dot(ab,ab),0.0,1.0);
        let arc_gain=(arc.y-arc.x)/length(ab);
        var q=vec2<f32>(dot(delta,n),dot(delta,binormal));var v=arc.x+dot(delta,tangent)*arc_gain;
        if continuous {
            let rotation=normalize(mix(procedural_data.items[base+5u],procedural_data.items[base+6u],t));
            n=anatomy_rotate(rotation,vec3<f32>(1.0,0.0,0.0));binormal=anatomy_rotate(rotation,vec3<f32>(0.0,1.0,0.0));tangent=anatomy_rotate(rotation,vec3<f32>(0.0,0.0,1.0));
            let radial=delta-ab*t;q=vec2<f32>(dot(radial,n),dot(radial,binormal));v=mix(arc.x,arc.y,t)+dot(radial,tangent)*arc_gain;
        }
        let powers=q*q*q*q;let w=powers.y/max(powers.x+powers.y,1e-16);
        let du=vec2<f32>(dot(dx,n),dot(dx,tangent)*arc_gain)/tile;let dv=vec2<f32>(dot(dy,n),dot(dy,tangent)*arc_gain)/tile;
        let du2=vec2<f32>(dot(dx,binormal),dot(dx,tangent)*arc_gain)/tile;let dv2=vec2<f32>(dot(dy,binormal),dot(dy,tangent)*arc_gain)/tile;
        let fp=max(length(du),length(dv));let fp2=max(length(du2),length(dv2));
        var sample=sample_mix(recipe_sample(info,vec2<f32>(q.x,v)/tile,fp),recipe_sample(info,vec2<f32>(q.y,v)/tile,fp2),1.0-w);
        if header.w>0.5 {
            let footprint=max(length(dx),length(dy));let g=mix(anatomy_grid(vec2<f32>(q.y,v),footprint),anatomy_grid(vec2<f32>(q.x,v),footprint),w);
            sample=SurfaceSample(vec4<f32>(vec3<f32>(g),0.9),vec4<f32>(0.0,0.5,1.0,1.0),vec4<f32>(0.0));
        } else {
            let age=max(0.0,mix(clocks.z,clocks.w,t));let m=1.0-exp(-age/evolution.x);let growth=max(0.0,mix(arc.z,arc.w,t));
            let gain=mix(young.w,mature.w,m)+growth_response*growth/(1.0+growth);
            sample.color=vec4<f32>(sample.color.xyz*mix(young.xyz,mature.xyz,m)*(1.0-0.35*evolution.w),clamp(sample.color.w-evolution.z+mix(evolution.y,evolution.z,m)-0.2*evolution.w,0.2,1.0));
            sample.response.y=clamp(0.5+(sample.response.y-0.5)*gain,0.0,1.0);
        }
        let distance=length(delta-ab*t)-mix(a.w,b.w,t);var weight=exp(-(distance-hits.distances.x)/header.z);
        if continuous {
            weight=exp(-(distances[i]-nearest)/header.z);
            if hits.count==3u && farthest-nearest>1e-5 {let window=clamp((farthest-distances[i])/(farthest-nearest),0.0,1.0);weight*=window*window*(3.0-2.0*window);}
        }
        result.color+=sample.color*weight;result.response+=sample.response*weight;result.emission+=sample.emission*weight;total+=weight;
    }
    var surface=SurfaceSample(result.color/max(total,1e-20),result.response/max(total,1e-20),result.emission/max(total,1e-20));
    let coating=u32(procedural_data.items[at+4u].z);
    if coating>0u && header.w<0.5 && procedural_data.items[at+coating].x>0.0 {
        let field=moss_field(at+coating,p,max(length(dx),length(dy)));let properties=procedural_data.items[at+coating+2u];
        let dark=procedural_data.items[at+coating+3u].xyz;let light=procedural_data.items[at+coating+4u].xyz;
        surface.color=vec4<f32>(mix(surface.color.xyz,mix(dark,light,field.w),field.x),mix(surface.color.w,properties.z,field.x));
        let bark=(surface.response.y-0.5)*procedural_data.items[at+4u].w;
        let height=mix(bark,field.z,field.x);
        surface.response.y=clamp(0.5+height/max(procedural_data.items[info.y+1u].x,1e-12),0.0,1.0);
        surface.response.x*=1.0-field.x;surface.response.w=mix(surface.response.w,0.97,field.x);surface.emission*=1.0-field.x;
    }
    return surface;
}

override prepared_anatomy: bool = false;
fn prepared_anatomy_sample(info:vec4<u32>,in:VertexOut,offset:vec2<f32>)->SurfaceSample {
    let tile=procedural_data.items[info.y].xy;
    let charts=array<vec4<f32>,3>(in.chart0,in.chart1,in.chart2);
    let weights=array<vec2<f32>,3>(in.chart_weights.xy,in.chart_weights.zw,in.chart_last.xy);
    let ages=array<vec2<f32>,3>(in.chart_last.zw,in.chart_evolution.xy,in.chart_evolution.zw);
    let young=procedural_data.items[info.w+1u];let mature=procedural_data.items[info.w+2u];let evolution=procedural_data.items[info.w+3u];
    var surface=SurfaceSample(vec4<f32>(0.),vec4<f32>(0.),vec4<f32>(0.));
    for(var j=0u;j<3u;j++) {
        let chart=charts[j]/tile.xyxy;
        let dx=dpdx(chart);let dy=dpdy(chart);
        let q=chart+dx*offset.x+dy*offset.y;
        let a=recipe_sample(info,q.xy,max(length(dx.xy),length(dy.xy)));
        let b=recipe_sample(info,q.zw,max(length(dx.zw),length(dy.zw)));
        let sum=weights[j].x+weights[j].y;
        var s=SurfaceSample(a.color*weights[j].x+b.color*weights[j].y,a.response*weights[j].x+b.response*weights[j].y,a.emission*weights[j].x+b.emission*weights[j].y);
        let m=ages[j].x;
        s.color=vec4<f32>(s.color.xyz*mix(young.xyz,mature.xyz,m)*(1.-0.35*evolution.w),clamp(s.color.w/max(sum,1e-20)-evolution.z+mix(evolution.y,evolution.z,m)-0.2*evolution.w,0.2,1.)*sum);
        s.response.y=clamp(0.5+(s.response.y/max(sum,1e-20)-0.5)*ages[j].y,0.,1.)*sum;
        surface.color+=s.color;surface.response+=s.response;surface.emission+=s.emission;
    }
    return surface;
}

fn recipe_uv(rest:vec3<f32>,domain:vec4<f32>)->vec2<f32> {
    if domain.z>2.5 && domain.z<3.5 {
        var angle=0.5;
        if dot(rest.xz,rest.xz)>1e-12 {angle=atan2(rest.x,rest.z)/6.283185307179586+0.5;}
        return vec2<f32>(angle,rest.y)/domain.xy;
    }
    return detail_coordinate(rest,domain.z)/domain.xy;
}
fn recipe_delta(rest:vec3<f32>,delta:vec3<f32>,domain:vec4<f32>)->vec2<f32> {
    if domain.z>2.5 && domain.z<3.5 {
        let radius2=max(dot(rest.xz,rest.xz),1e-12);
        let angular=(rest.z*delta.x-rest.x*delta.z)/(6.283185307179586*radius2);
        return vec2<f32>(angular,delta.y)/domain.xy;
    }
    return recipe_uv(delta,domain);
}
fn triplanar_sample(info:vec4<u32>,p:vec3<f32>,w:vec3<f32>,footprint:f32)->SurfaceSample {
    let tile=procedural_data.items[info.y].xy;let a=recipe_sample(info,p.yz/tile,footprint);let b=recipe_sample(info,p.xz/tile,footprint);let c=recipe_sample(info,p.xy/tile,footprint);
    return SurfaceSample(a.color*w.x+b.color*w.y+c.color*w.z,a.response*w.x+b.response*w.y+c.response*w.z,a.emission*w.x+b.emission*w.y+c.emission*w.z);
}
struct ReliefSample { sample:SurfaceSample, shadow:f32, uv:vec2<f32>, };
fn recipe_relief(info:vec4<u32>,uv:vec2<f32>,footprint:f32,du:vec3<f32>,dv:vec3<f32>,normal:vec3<f32>,world:vec3<f32>,geometric_pixel:f32)->ReliefSample {
    let domain=procedural_data.items[info.y];let relief=procedural_data.items[info.y+1u];
    var q=uv;var depth=0.0;var shadow=1.0;
    let delta=globals.view_position.xyz-world;let view=delta/max(length(delta),0.000001);let no_v=dot(normal,view);
    // Bound grazing offsets and fade when the relief becomes subpixel.
    var world_pixel=footprint*max(domain.x,domain.y);
    if domain.z>2.5 {world_pixel=geometric_pixel;}
    let depth_scale=max(relief.x,1e-8);
    let fade=smoothstep(0.05,0.2,no_v)*(1.0-smoothstep(depth_scale*0.5,depth_scale*2.0,max(world_pixel,1e-8)));
    if domain.w>0.5 && domain.w<1.5 && relief.x>0.0 && fade>0.0 {
        let shift=vec2<f32>(dot(du,view),dot(dv,view))*relief.x*fade/max(no_v,0.05);
        let steps=u32(relief.y);var previous=0.0;var lo=0.0;var hi=1.0;
        for(var i=0u;i<=48u;i++) {
            if i>steps {break;}
            let t=f32(i)/f32(steps);let h=1.0-recipe_sample(info,uv-shift*t,footprint).response.y;
            if t>=h {lo=previous;hi=t;break;}previous=t;
        }
        for(var i=0u;i<4u;i++) {let t=(lo+hi)*0.5;let h=1.0-recipe_sample(info,uv-shift*t,footprint).response.y;if t>=h {hi=t;}else{lo=t;}}
        depth=(lo+hi)*0.5;q=uv-shift*depth;
        let light=-normalize(globals.light.xyz);let no_l=dot(normal,light);
        if relief.z>0.0 && no_l>0.05 {
            let light_shift=vec2<f32>(dot(du,light),dot(dv,light))*relief.x*fade/no_l;
            for(var i=1u;i<=24u;i++) {
                if i>u32(relief.z) {break;}
                let t=depth*f32(i)/relief.z;let surface_depth=1.0-recipe_sample(info,q+light_shift*t,footprint).response.y;
                if surface_depth+0.015<depth-t {shadow=1.0-fade;break;}
            }
        }
    }
    return ReliefSample(recipe_sample(info,q,footprint),shadow,q);
}

struct ShadedSurface { color: vec3<f32>, normal: vec3<f32>, base: vec3<f32>, };
fn shade_surface(in: VertexOut, front: bool) -> ShadedSurface {
    var material = materials.items[in.material];
    let px = dpdx(in.world);
    let py = dpdy(in.world);
    let geometric_normal = normalize(in.normal)*select(-1.0,1.0,front);
    var recipe_color=vec3<f32>(1.0);var recipe_height=0.0;var recipe_shadow=1.0;var recipe_ao=1.0;
    var pixel_gradient=vec2<f32>(0.0);var pixel_normals=false;
    let rest_dx=dpdx(in.rest);let rest_dy=dpdy(in.rest);
    if !uniform_material && material.procedural.x>0u {
        let info=material.procedural;let domain=procedural_data.items[info.y];
        let uv=recipe_uv(in.rest,domain);let dx=recipe_delta(in.rest,rest_dx,domain);let dy=recipe_delta(in.rest,rest_dy,domain);
        let r1=cross(py,geometric_normal);let r2=cross(geometric_normal,px);let determinant=dot(px,r1);
        var du=vec3<f32>(0.0);var dv=vec3<f32>(0.0);
        if abs(determinant)>1e-12 {du=(dx.x*r1+dy.x*r2)/determinant;dv=(dx.y*r1+dy.y*r2)/determinant;}
        var result:ReliefSample;
        if prepared_anatomy {
            let sample=prepared_anatomy_sample(info,in,vec2<f32>(0.));result=ReliefSample(sample,1.0,uv);
            if procedural_data.items[info.y+1u].w>0.5 {
                let depth=procedural_data.items[info.y+1u].x;
                pixel_gradient=vec2<f32>(prepared_anatomy_sample(info,in,vec2<f32>(1.,0.)).response.y-sample.response.y,prepared_anatomy_sample(info,in,vec2<f32>(0.,1.)).response.y-sample.response.y)*depth;
                pixel_normals=true;
            }
        } else if anatomical_material && info.w>0u {
            let hits=anatomy_hits(info.w,in.rest);let sample=anatomy_sample(info,in.rest,rest_dx,rest_dy,hits);
            result=ReliefSample(sample,1.0,uv);
            let height=procedural_data.items[info.y+1u].x;
            if procedural_data.items[info.y+1u].w>0.5 {pixel_gradient=vec2<f32>(anatomy_sample(info,in.rest+rest_dx,rest_dx,rest_dy,hits).response.y-sample.response.y,
                anatomy_sample(info,in.rest+rest_dy,rest_dx,rest_dy,hits).response.y-sample.response.y)*height;
            pixel_normals=true;}
        } else if domain.z>3.5 {
            var rest_normal=abs(cross(rest_dx,rest_dy));if max(rest_normal.x,max(rest_normal.y,rest_normal.z))<1e-25 {rest_normal=abs(geometric_normal);}let scaled=rest_normal/max(max(rest_normal.x,max(rest_normal.y,rest_normal.z)),1e-30);let powers=scaled*scaled*scaled*scaled;let weights=powers/max(dot(powers,vec3<f32>(1.0)),1e-30);
            let fp=max(length(rest_dx),length(rest_dy))/min(domain.x,domain.y);
            let sample=triplanar_sample(info,in.rest,weights,fp);result=ReliefSample(sample,1.0,uv);
            let depth=procedural_data.items[info.y+1u].x;
            if procedural_data.items[info.y+1u].w>0.5 {pixel_gradient=vec2<f32>(triplanar_sample(info,in.rest+rest_dx,weights,fp).response.y-sample.response.y,triplanar_sample(info,in.rest+rest_dy,weights,fp).response.y-sample.response.y)*depth;
            pixel_normals=true;}
        } else {result=recipe_relief(info,uv,max(length(dx),length(dy)),du,dv,geometric_normal,in.world,max(length(px),length(py)));}
        recipe_color=result.sample.color.xyz;recipe_height=result.sample.response.y*procedural_data.items[info.y+1u].x;
        let relief=procedural_data.items[info.y+1u];
        if info.w==0u && domain.z<3.5 && relief.w>0.5 && relief.x>0.0 {
            let fp=max(length(dx),length(dy));
            pixel_gradient=vec2<f32>(recipe_sample(info,result.uv+dx,fp).response.y-result.sample.response.y,
                recipe_sample(info,result.uv+dy,fp).response.y-result.sample.response.y)*relief.x;
            pixel_normals=true;
        }
        if relief.y<0.0 {recipe_color*=1.0+relief.y*(1.0-clamp(in.material_projection_weights.x,0.0,1.0));}
        recipe_shadow=result.shadow;recipe_ao=result.sample.response.w;
        material.low.w=result.sample.color.w;material.high.w=result.sample.response.x;material.response.w=0.04;
        material.response.x=result.sample.emission.w;material.emissive=vec4<f32>(result.sample.emission.xyz,0.0);
    }
    var recipe_hx=dpdx(recipe_height);var recipe_hy=dpdy(recipe_height);
    if pixel_normals {recipe_hx=pixel_gradient.x;recipe_hy=pixel_gradient.y;}
    var tint = vec3<f32>(1.0);
    var hx = 0.0;
    var hy = 0.0;
    if !uniform_material && !fixed_recipe && !prepared_anatomy {
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
    hx+=recipe_hx;hy+=recipe_hy;
    tint*=recipe_color;
    var normal = normalize(in.normal);
    if (material.response.x > 0.0 || material.surface.x >= 1.0 || material.surface.w > 0.5) && !front { normal = -normal; }
    let r1 = cross(py, normal);
    let r2 = cross(normal, px);
    let determinant = dot(px, r1);
    if abs(determinant) > 0.0000000001 {
        normal = normalize(abs(determinant) * normal - sign(determinant) * (hx*r1 + hy*r2));
    }
    let light = -normalize(globals.light.xyz);
    let diffuse = max(dot(normal, light), 0.0) + material.response.x * max(dot(-normal, light), 0.0);
    var visible: f32;
    if (u32(material.attenuation.w) & 4u) != 0u {
        visible = recipe_shadow;
    } else if cull_zero_light_shadow {
        // Test the bumped/flipped normal and retain thin-sheet backlighting.
        let needed = dot(normal, light)>0.0 || (material.response.x>0.0 && dot(-normal,light)>0.0);
        visible = selected_visibility(in.world, needed)*recipe_shadow;
    } else { visible = visibility(in.world)*recipe_shadow; }
    var base = in.color * tint;
    if globals.atmosphere_detail.w>0.0 && globals.planet_optics.x==0.0 && in.material>0u && in.material<=u32(globals.planet_space.w) {
        let rel=in.world-globals.planet_eye_high.xyz;
        let p=vec3<f32>(dot(rel,globals.planet_axis0.xyz),dot(rel,globals.planet_axis1.xyz),dot(rel,globals.planet_axis2.xyz));
        let radius=globals.planet_radial.w;
        let altitude=p.y+globals.planet_radial.x+dot(p.xz,p.xz)/(2.0*radius);
        let up=normalize(globals.planet_axis1.xyz+(p.x*globals.planet_axis0.xyz+p.z*globals.planet_axis2.xyz)/radius);
        let slope=smoothstep(globals.planet_axis1.w,globals.planet_axis0.w,abs(dot(normal,up)));
        let region=1.0-smoothstep(globals.planet_radial.z*0.9,globals.planet_radial.z,length(p.xz));
        let height=1.0-smoothstep(globals.planet_radial.y-200.0,globals.planet_radial.y,altitude);
        // Distant meadow coverage uses the same regional mask as the root field.
        // Individual leaves remain geometry near the player; no opaque grass wall.
        base=mix(base,globals.planet_space.xyz,slope*region*height);
    }
    let ambient_visibility=world_ambient_visibility(in.world,normalize(in.normal));
    var color = base * (globals.light.w * ambient_visibility + (1.0 - globals.light.w) * diffuse * visible);
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
        color = base * globals.light.w * ambient_visibility + (1.0-globals.light.w) * visible
            * mix(dielectric_direct, metal_direct, material.high.w);
    } else if material.response.w > 0.0 {
        let response = dielectric(normal, light, in.world, material.low.w, material.response.w);
        color += (1.0 - globals.light.w) * visible * (vec3<f32>(response.y) - base * response.x);
    }
    if !uniform_material && material.procedural.x>0u {color-=base*globals.light.w*ambient_visibility*(1.0-recipe_ao);}
    if any(globals.direct_light.xyz != vec3<f32>(1.0)) {
        let ambient_color = base * globals.light.w * recipe_ao * ambient_visibility;
        color = ambient_color + (color-ambient_color) * globals.direct_light.xyz;
    }
    if globals.environment.x > 0.0 {
        let response=environment_response(base,normal,in.world,material);
        color += globals.environment.x * recipe_ao * (response.diffuse*ambient_visibility+response.specular);
    }
    if material.surface.w > 0.5 {
        // Thin-sheet glass has reflection, no diffuse lobe or invented ambient.
        let direct = dielectric(normal, light, in.world, material.low.w, material.response.w).y;
        if globals.direct_light.w > 0.5 {
            // Preserve the old scalar multiplication order for exact white renders.
            color = vec3<f32>((1.0-globals.light.w) * globals.environment.z * visible * direct);
        } else {
            color = ((1.0-globals.light.w) * visible * direct) * globals.direct_light.xyz;
        }
        if globals.environment.x > 0.0 {
            color += globals.environment.x * environment_light(vec3<f32>(0.0), normal, in.world, material);
        }
    }
    if any(material.emissive.xyz > vec3<f32>(0.0)) { color += material.emissive.xyz; }
    // World-space metre grid on the floor, generated by the renderer.
    let coord = in.world.xz;
    let grid = abs(fract(coord - 0.5) - 0.5) / max(fwidth(coord), vec2<f32>(0.0001));
    let line = 1.0 - min(min(grid.x, grid.y), 1.0);
    if globals.environment.w < 0.5 && in.world.y < 0.02 && in.world.y > -0.02 && in.normal.y > 0.9 {
        color = mix(color, color + vec3<f32>(0.055, 0.075, 0.08), line * 0.65);
    }
    if globals.atmosphere.w > 0.0 || globals.atmosphere_sky.w > 0.0 {
        if globals.planet_optics.x > 0.0 && in.material>0u && in.material<=u32(globals.planet_space.w) {
            color=planet_terrain_air(color,in.world,in.rest);
        }else if material.surface.w > 0.5 {
            let alpha = 1.0-dot(glass_transmission(base,normal,in.world,material),vec3<f32>(1.0/3.0));
            color = atmosphere_surface(color/max(alpha,0.000001), in.world)*alpha;
        } else {
            color = atmosphere_surface(color, in.world);
        }
    }
    return ShadedSurface(color, normal, base);
}
fn surface_opacity(rest: vec3<f32>, material: SurfaceMaterial) -> f32 {
    if uniform_material { return material.surface.y; }
    let p = rest * material.coverage.frequency.xyz;
    let footprint = max(length(dpdx(p)), length(dpdy(p)));
    let domain0=procedural_data.items[material.procedural.y];
    let domain=vec4<f32>(max(domain0.xy,vec2<f32>(0.000001)),domain0.zw);
    let uv=recipe_uv(rest,domain);
    let rest_dx=dpdx(rest);let rest_dy=dpdy(rest);
    let dx=recipe_delta(rest,rest_dx,domain);let dy=recipe_delta(rest,rest_dy,domain);
    var opacity = material.surface.y;
    // Opaque is validated to full coverage. Derivatives above remain quad-uniform;
    // skip a second mineral evaluation in the color pass and all opaque shadows.
    if material.surface.x==0.0 {return opacity;}
    if material.procedural.x>0u {
        if anatomical_material && material.procedural.w>0u {opacity*=anatomy_sample(material.procedural,rest,rest_dx,rest_dy,anatomy_hits(material.procedural.w,rest)).response.z;}
        else {opacity*=recipe_sample(material.procedural,uv,max(length(dx),length(dy))).response.z;}
    }
    if material.coverage.frequency.w > 0.5 {
        let field = pattern_field(p, footprint, material.coverage.control, material.coverage.response);
        opacity *= mix(material.coverage.low.x, material.coverage.high.x, field);
    }
    return clamp(opacity, 0.0, 1.0);
}
@fragment fn fs_main(in: VertexOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let shaded = shade_surface(in, front);
    let material = materials.items[in.material];
    let opacity = surface_opacity(in.rest, material);
    if outside_horizon(in.world) { discard; }
    if material.surface.x == 1.0 && opacity < material.surface.z { discard; }
    return vec4<f32>(shaded.color, 1.0);
}
// Selected only for validated opaque materials with no horizon discard. Keeping
// discard out of the reachable shader permits ordinary hardware early depth.
@fragment fn fs_opaque(in:VertexOut,@builtin(front_facing) front:bool)->@location(0) vec4<f32> {
    return vec4<f32>(shade_surface(in,front).color,1.0);
}
@fragment fn fs_shadow(in: ShadowOut) {
    let material = materials.items[in.material];
    if (u32(material.attenuation.w) & 2u) != 0u { discard; }
    let opacity = surface_opacity(in.rest, material);
    if material.surface.x >= 2.0 || (material.surface.x == 1.0 && opacity < material.surface.z) { discard; }
}
struct TransparentOut {
    @location(0) accumulation: vec4<f32>,
    @location(1) transmission: vec4<f32>,
    @location(2) additive: vec4<f32>,
};
fn transparent_surface(in: VertexOut, front: bool, multiplier:f32, tint:vec3<f32>) -> TransparentOut {
    let shaded = shade_surface(in, front);
    let material = materials.items[in.material];
    let coverage = surface_opacity(in.rest, material)*multiplier;
    if outside_horizon(in.world) { discard; }
    var out: TransparentOut;
    out.accumulation = vec4<f32>(0.0);
    out.transmission = vec4<f32>(1.0);
    out.additive = vec4<f32>(0.0);
    if material.surface.x == 3.0 {
        out.additive = vec4<f32>(min(shaded.color * tint * coverage, vec3<f32>(64.0)), 0.0);
        return out;
    }
    var transmission = vec3<f32>(1.0-coverage);
    var color = shaded.color * tint * coverage;
    if material.surface.w > 0.5 {
        transmission = mix(vec3<f32>(1.0), glass_transmission(shaded.base,shaded.normal,in.world,material), coverage);
    }
    let alpha = clamp(1.0-dot(transmission, vec3<f32>(1.0/3.0)), 0.0, 1.0);
    // A bounded world-distance weight avoids dependence on the near/far projection.
    let weight = clamp(1.0/(1.0+0.01*length(in.world-globals.view_position.xyz)), 0.01, 1.0);
    out.accumulation = vec4<f32>(min(color, vec3<f32>(64.0))*weight, alpha*weight);
    out.transmission = vec4<f32>(transmission, 1.0-alpha);
    return out;
}

@fragment fn fs_transparent(in:VertexOut,@builtin(front_facing) front:bool)->TransparentOut {return transparent_surface(in,front,1.,vec3<f32>(1.));}

// Diagnostic guides keep stable colors independent of gameplay lighting.
@fragment fn fs_guide(in: VertexOut) -> @location(0) vec4<f32> {
    if outside_horizon(in.world) { discard; }
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
fn atmosphere_vertex(index:u32,reverse:bool) -> AtmosphereOut {
    let points = array<vec2<f32>, 3>(vec2<f32>(-1.0,-1.0), vec2<f32>(3.0,-1.0), vec2<f32>(-1.0,3.0));
    let clip = vec4<f32>(points[index], select(1.0,0.0,reverse), 1.0);
    var out: AtmosphereOut;
    out.clip = clip;
    out.ray = globals.atmosphere_forward.xyz + clip.x * globals.atmosphere_right.xyz + clip.y * globals.atmosphere_up.xyz;
    return out;
}
// Exact path through a homogeneous finite spherical shell, clipped by the first
// rendered surface. Normalized radius avoids subtracting astronomical squares.
@vertex fn vs_atmosphere(@builtin(vertex_index) index:u32)->AtmosphereOut {return atmosphere_vertex(index,globals.planet_optics.x>0.0);}
@vertex fn vs_atmosphere_reverse(@builtin(vertex_index) index:u32)->AtmosphereOut {return atmosphere_vertex(index,true);}
fn planet_air_path(ray:vec3<f32>,limit:f32)->f32 {
    let eta=globals.planet_optics.y;
    let top=globals.planet_optics.z;
    let radius=1.0+eta;
    let b=radius*dot(globals.planet_radial.xyz,ray);
    var entry=0.0;
    var exit=0.0;
    if radius>=2.0 {
        let perpendicular=radius*length(cross(globals.planet_radial.xyz,ray));
        let outer=1.0+top;
        if perpendicular>outer || b>=0.0 {return 0.0;}
        let half=sqrt(max(0.0,outer*outer-perpendicular*perpendicular));
        entry=-b-half;exit=-b+half;
    } else {
        let c=(eta-top)*(2.0+eta+top);
        let disc=b*b-c;
        if disc<0.0 {return 0.0;}
        let root=sqrt(disc);
        let q=-b-select(-root,root,b>=0.0);
        if abs(q)<1e-20 {return 0.0;}
        entry=min(q,c/q);exit=max(q,c/q);
    }
    return max(0.0,min(limit,exit)-max(0.0,entry));
}
// Generated terrain carries the canonical unit direction *4096 in rest.
// Linear triangle chords sit inside the curved source. Recover interpolated
// radial height to avoid interpreting LOD sagitta as kilometres of extra air.
// This changes optics only: the visible terrain, depth and physics stay meshes.
fn planet_terrain_air(surface:vec3<f32>,world:vec3<f32>,rest:vec3<f32>)->vec3<f32> {
    let normalized_world=world/globals.planet_radial.w;
    let distance=length(normalized_world);
    let ray=normalized_world/max(distance,1e-20);
    let eye_radius=1.0+globals.planet_optics.y;
    let global=normalized_world+eye_radius*globals.planet_radial.xyz;
    let radius=length(global)/max(length(rest)/4096.0,0.1);
    let b=eye_radius*dot(globals.planet_radial.xyz,ray);
    let perpendicular=eye_radius*length(cross(globals.planet_radial.xyz,ray));
    let half=sqrt(max(0.0,radius*radius-perpendicular*perpendicular));
    let near=-b-half;
    let far=-b+half;
    let hit=select(far,near,abs(near-distance)<=abs(far-distance));
    // Root selection near tangency can amplify f32 error. Enforce the same
    // displacement cap consumed by the CPU certificate; otherwise keep the mesh.
    let limit=select(distance,hit,hit>=0.0 && abs(hit-distance)<=globals.planet_eye_high.w);
    let transmission=exp(-planet_air_path(ray,limit)*globals.planet_optics.w);
    return mix(atmosphere_color(ray),surface,transmission);
}
@fragment fn fs_atmosphere(in: AtmosphereOut) -> @location(0) vec4<f32> {
    let direction = normalize(in.ray);
    if globals.planet_optics.x > 0.0 {
        let transmission=exp(-planet_air_path(direction,1e30)*globals.planet_optics.w);
        return vec4<f32>(mix(atmosphere_color(direction),globals.planet_space.xyz,transmission),1.0);
    }
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
    if outside_horizon(in.world) { discard; }
    return vec4<f32>(atmosphere_surface(in.color,in.world),1.0);
}
fn outside_horizon(world:vec3<f32>) -> bool {
    let limit=globals.atmosphere_detail.z;
    return limit>0.0 && distance(world,globals.view_position.xyz)>limit;
}
fn atmosphere_surface(surface: vec3<f32>, world: vec3<f32>) -> vec3<f32> {
    let ray = world - globals.view_position.xyz;
    let distance = length(ray);
    if globals.planet_optics.x > 0.0 {
        let transmission=exp(-planet_air_path(ray/max(distance,0.000001),distance/globals.planet_radial.w)*globals.planet_optics.w);
        return mix(atmosphere_color(ray/max(distance,0.000001)),surface,transmission);
    }
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
    out.color = color*i.tint.xyz; out.rest = rest; out.material = i.material; out.material_projection_weights = weights;
    return out;
}
fn shared_shadow(position: vec3<f32>, rest: vec3<f32>, material: u32, i: StaticInstance) -> ShadowOut {
    var out: ShadowOut;
    out.clip = globals.shadow_projection * vec4<f32>(shared_world(i,position),1.0);
    out.rest = rest; out.material = material;
    return out;
}
@vertex fn vs_shared_shadow(@location(0) position: vec3<f32>, @location(3) rest: vec3<f32>, @location(4) material: u32, i: StaticInstance) -> ShadowOut {
    return shared_shadow(position, rest, i.material, i);
}
@vertex fn vs_shared_shadow_compact(@location(0) position: vec3<f32>, @location(3) rest: vec3<f32>, i: StaticInstance) -> ShadowOut {
    return shared_shadow(position, rest, i.material, i);
}
@vertex fn vs_shared_shadow_minimal(@location(0) position: vec3<f32>, i: StaticInstance) -> ShadowOut {
    return shared_shadow(position, position, i.material, i);
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

@vertex fn vs_shared_anatomy(@location(0) position:vec3<f32>,@location(1) normal:vec3<f32>,
    @location(3) rest:vec3<f32>,@location(5) weights:vec3<f32>,
    @location(12) chart0:vec4<f32>,@location(13) chart1:vec4<f32>,@location(14) chart2:vec4<f32>,
    @location(15) blend:vec4<f32>,@location(2) last:vec4<f32>,@location(4) evolution:vec4<f32>,i:StaticInstance)->VertexOut {
    var out:VertexOut;
    out.world=shared_world(i,position);out.clip=globals.view_projection*vec4<f32>(out.world,1.);
    out.normal=normalize(mat3x3<f32>(i.x.xyz,i.y.xyz,i.z.xyz)*normal);
    out.color=i.tint;out.material=i.material;out.rest=rest;out.material_projection_weights=weights;
    out.chart0=chart0;out.chart1=chart1;out.chart2=chart2;out.chart_weights=blend;out.chart_last=last;out.chart_evolution=evolution;
    return out;
}

// Keep the main clip coordinates (including its near/far clipping). Only the
// depth stored by the auxiliary pass is reversed; this cannot add near-plane
// coverage that the actual surface does not have.
struct OcclusionOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) eye_depth: f32,
    @location(1) world: vec3<f32>,
};
@vertex fn vs_shared_occlusion(@location(0) position: vec3<f32>, i: StaticInstance) -> OcclusionOut {
    var out: OcclusionOut;
    out.clip = globals.view_projection * vec4<f32>(shared_world(i,position),1.0);
    out.eye_depth = out.clip.w;
    out.world = shared_world(i,position);
    return out;
}
@fragment fn fs_occlusion(in: OcclusionOut) -> @builtin(frag_depth) f32 {
    if outside_horizon(in.world) { discard; }
    return globals.view_position.w / in.eye_depth;
}

@vertex fn vs_occlusion(@location(0) position: vec3<f32>) -> OcclusionOut {
    var out: OcclusionOut;
    out.clip = globals.view_projection * vec4<f32>(position,1.0);
    out.eye_depth = out.clip.w;
    out.world = position;
    return out;
}
struct CoverageDepthOut {
    @builtin(position) clip:vec4<f32>,
    @location(0) rest:vec3<f32>,
    @location(1) @interpolate(flat) material:u32,
    @location(2) eye_depth:f32,
    @location(3) world:vec3<f32>,
};
fn coverage_depth(position:vec3<f32>,rest:vec3<f32>,i:StaticInstance)->CoverageDepthOut{
    var out:CoverageDepthOut;
    out.clip=globals.view_projection*vec4<f32>(shared_world(i,position),1.0);
    out.rest=rest;out.material=i.material;out.eye_depth=out.clip.w;
    out.world=shared_world(i,position);return out;
}
@vertex fn vs_shared_coverage_depth(@location(0) position:vec3<f32>,@location(3) rest:vec3<f32>,i:StaticInstance)->CoverageDepthOut{
    return coverage_depth(position,rest,i);
}
@vertex fn vs_shared_position_depth(@location(0) position:vec3<f32>,i:StaticInstance)->CoverageDepthOut{
    return coverage_depth(position,position,i);
}
@fragment fn fs_coverage_depth(in:CoverageDepthOut)->@builtin(frag_depth) f32{
    let material=materials.items[in.material];
    let opacity=surface_opacity(in.rest,material);
    if outside_horizon(in.world) {discard;}
    if material.surface.x>=2.0 || (material.surface.x==1.0 && opacity<material.surface.z){discard;}
    return globals.view_position.w/in.eye_depth;
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

// Same full procedural material; only redundant static coordinates are reconstructed.
@vertex fn vs_shared_position_coordinates(@location(0) position: vec3<f32>, @location(1) normal:vec3<f32>, i:StaticInstance) -> VertexOut {
    var out:VertexOut;
    out.world=shared_world(i,position);
    out.clip=globals.view_projection * vec4<f32>(out.world,1.0);
    out.normal=normalize(mat3x3<f32>(i.x.xyz,i.y.xyz,i.z.xyz)*normal);
    out.color=i.tint; out.material=i.material;
    out.rest=position; out.material_projection_weights=abs(normal);
    return out;
}

// All tiles share the same split geocentric coordinates and observer constants.
// Subtract before rescaling: close border vertices reconstruct bit-identically.
fn planet_vertex(high:vec3<f32>,normal:vec3<f32>,low:vec3<f32>,rest:vec3<f32>,weights:vec3<f32>,i:StaticInstance)->VertexOut {
    let axes=mat3x3<f32>(globals.planet_axis0.xyz,globals.planet_axis1.xyz,globals.planet_axis2.xyz);
    let delta=(high-globals.planet_eye_high.xyz)+(low-globals.planet_eye_low.xyz);
    var out:VertexOut;
    out.world=axes*(delta*globals.planet_radial.w);
    out.clip=globals.view_projection*vec4<f32>(out.world,1.0);
    out.normal=normalize(axes*normal);
    out.color=i.tint;out.rest=rest;out.material=i.material;out.material_projection_weights=weights;
    return out;
}

// No projection weights are fetched when the material has no detail channel.
@vertex fn vs_shared_planet_compact(@location(0) high:vec3<f32>,@location(1) normal:vec3<f32>,
    @location(3) low:vec3<f32>,@location(5) rest:vec3<f32>,i:StaticInstance)->VertexOut {
    return planet_vertex(high,normal,low,rest,vec3<f32>(0.0),i);
}

@vertex fn vs_shared_planet(@location(0) high:vec3<f32>,@location(1) normal:vec3<f32>,
    @location(3) low:vec3<f32>,@location(5) rest:vec3<f32>,@location(12) weights:vec3<f32>,i:StaticInstance)->VertexOut {
    return planet_vertex(high,normal,low,rest,weights,i);
}

struct ParticleOut {
    @builtin(position) clip:vec4<f32>,
    @location(0) world:vec3<f32>,
    @location(1) normal:vec3<f32>,
    @location(2) uv:vec2<f32>,
    @location(3) @interpolate(flat) material:u32,
    @location(4) @interpolate(flat) tint:vec4<f32>,
    @location(5) @interpolate(flat) softness:f32,
};
@vertex fn vs_particle(@builtin(vertex_index) index:u32,
    @location(0) center_size:vec4<f32>,@location(1) velocity_angle:vec4<f32>,
    @location(2) color:vec4<f32>,@location(3) shape:vec4<f32>,@location(4) material:vec4<u32>)->ParticleOut {
    let corners=array<vec2<f32>,6>(vec2<f32>(-1.,-1.),vec2<f32>(1.,-1.),vec2<f32>(1.,1.),vec2<f32>(-1.,-1.),vec2<f32>(1.,1.),vec2<f32>(-1.,1.));
    let uv=corners[index];
    let right=normalize(globals.atmosphere_right.xyz);
    let up=normalize(globals.atmosphere_up.xyz);
    var angle=velocity_angle.w;
    let projected=vec2<f32>(dot(velocity_angle.xyz,right),dot(velocity_angle.xyz,up));
    if shape.z>0.5 && dot(projected,projected)>0.000001 {angle+=atan2(projected.y,projected.x)-1.57079632679;}
    let cs=cos(angle);let sn=sin(angle);
    let local=vec2<f32>(uv.x*shape.x,uv.y)*center_size.w*0.5;
    let offset=vec2<f32>(local.x*cs-local.y*sn,local.x*sn+local.y*cs);
    var out:ParticleOut;
    out.world=center_size.xyz+right*offset.x+up*offset.y;
    out.clip=globals.view_projection*vec4<f32>(out.world,1.);
    out.normal=-normalize(globals.atmosphere_forward.xyz);
    out.uv=uv;out.material=material.x;out.tint=color;out.softness=shape.y;
    return out;
}
@fragment fn fs_particle(in:ParticleOut)->TransparentOut {
    let radius=length(in.uv);
    let edge=max(fwidth(radius),0.0001);
    let width=max(in.softness,edge);
    let profile=1.-smoothstep(1.-width,1.,radius);
    var surface:VertexOut;
    surface.clip=in.clip;surface.world=in.world;surface.normal=in.normal;
    surface.color=vec3<f32>(1.);surface.rest=vec3<f32>((in.uv+vec2<f32>(1.))*0.5,0.);
    surface.material=in.material;surface.material_projection_weights=vec3<f32>(1.,0.,0.);
    return transparent_surface(surface,true,profile*in.tint.a,in.tint.rgb);
}
