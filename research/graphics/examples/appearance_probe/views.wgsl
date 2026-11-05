struct Uniforms {
    matrix: mat4x4<f32>, inverse: mat4x4<f32>,
    source0: mat4x4<f32>, source1: mat4x4<f32>,
    view: vec4<f32>, light: vec4<f32>,
    // view coordinate, light coordinate, angular count, mode (0 planar, 1 height, 2 bake height)
    settings: vec4<f32>, rect: vec4<f32>,
};
@group(0) @binding(0) var<uniform> g: Uniforms;
@group(0) @binding(1) var atlas: texture_2d_array<f32>;
@group(0) @binding(2) var filtering: sampler;
struct GeometryOut { @builtin(position) position: vec4<f32>, @location(0) color: vec3<f32>, @location(1) height: f32 };
fn shade(n: vec3<f32>, color: vec3<f32>) -> vec3<f32> {
    let ndl = max(dot(n,g.light.xyz),0.0);
    let h = normalize(g.light.xyz+g.view.xyz);
    let glossy = color.b > color.r;
    let exponent = select(12.0,96.0,glossy);
    let amplitude = select(0.2,2.0,glossy);
    return color*(0.03+ndl)+vec3<f32>(pow(max(dot(n,h),0.0),exponent)*amplitude*ndl);
}
@vertex fn geometry(@location(0) p: vec3<f32>, @location(1) n: vec3<f32>, @location(2) c: vec3<f32>, @location(3) tile: u32) -> GeometryOut {
    var o: GeometryOut;
    o.position=g.matrix*vec4<f32>(p,1.0); o.color=shade(n,c); o.height=p.y; return o;
}
@fragment fn surface(i: GeometryOut) -> @location(0) vec4<f32> {
    if g.settings.w > 1.5 { return vec4<f32>(vec3<f32>(i.height),1.0); }
    return vec4<f32>(i.color,1.0);
}
struct Screen { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn screen(@builtin(vertex_index) id: u32) -> Screen {
    let corners=array<vec2<f32>,6>(vec2<f32>(0.0,0.0),vec2<f32>(1.0,0.0),vec2<f32>(0.0,1.0),vec2<f32>(0.0,1.0),vec2<f32>(1.0,0.0),vec2<f32>(1.0,1.0));
    let uv=mix(g.rect.xy,g.rect.zw,corners[id]); var o: Screen;
    o.position=vec4<f32>(uv*vec2<f32>(2.0,-2.0)+vec2<f32>(-1.0,1.0),0.0,1.0);o.uv=uv;return o;
}
fn project(p: vec3<f32>, m: mat4x4<f32>) -> vec2<f32> {
    let clip=m*vec4<f32>(p,1.0);return clip.xy/clip.w*vec2<f32>(0.5,-0.5)+vec2<f32>(0.5);
}
fn lookup(uv: vec2<f32>, v: i32, l: f32) -> vec4<f32> {
    if any(uv<vec2<f32>(0.0)) || any(uv>vec2<f32>(1.0)) {return vec4<f32>(0.0);}
    // Half-texel clamp prevents adjacent light tiles bleeding into one another.
    let q=clamp(uv,vec2<f32>(0.5/128.0),vec2<f32>(1.0-0.5/128.0));
    return textureSampleLevel(atlas,filtering,vec2<f32>((l+q.x)/(g.settings.z+1.0),q.y),v,0.0);
}
fn reconstruct(origin: vec3<f32>, ray: vec3<f32>, m: mat4x4<f32>, v: i32) -> vec4<f32> {
    var p=origin+ray*((0.05-origin.y)/ray.y);
    if g.settings.w > 0.5 {
        for(var k=0u;k<3u;k++) {
            let h=lookup(project(p,m),v,g.settings.z);
            if h.a>0.05 {p=origin+ray*((h.r/h.a-origin.y)/ray.y);}
        }
    }
    let uv=project(p,m);let l=floor(g.settings.y);let next=(i32(l)+1)%i32(g.settings.z);
    return mix(lookup(uv,v,l),lookup(uv,v,f32(next)),fract(g.settings.y));
}
@fragment fn proxy(i: Screen) -> @location(0) vec4<f32> {
    // Absolute fragment coordinates make reconstruction independent of proxy triangulation.
    let xy=(i.position.xy/g.view.w)*vec2<f32>(2.0,-2.0)+vec2<f32>(-1.0,1.0);
    let near=g.inverse*vec4<f32>(xy,0.0,1.0);let far=g.inverse*vec4<f32>(xy,1.0,1.0);
    let origin=near.xyz/near.w;let ray=normalize(far.xyz/far.w-origin);
    let v=i32(floor(g.settings.x));let next=(v+1)%i32(g.settings.z);
    return mix(reconstruct(origin,ray,g.source0,v),reconstruct(origin,ray,g.source1,next),fract(g.settings.x));
}
