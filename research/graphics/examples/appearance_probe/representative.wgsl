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
// Direct GGX/Schlick material evaluation shared by source and offline bake.
struct GeometryOut {
    @builtin(position) position: vec4<f32>, @location(0) color: vec3<f32>,
    @location(1) height: f32, @location(2) normal: vec3<f32>,
    @location(3) @interpolate(flat) material: u32,
};
@vertex fn geometry(@location(0) p: vec3<f32>, @location(1) n: vec3<f32>, @location(2) c: vec3<f32>, @location(3) material: u32) -> GeometryOut {
    var o: GeometryOut; o.position=g.matrix*vec4<f32>(p,1.0);
    o.color=c; o.height=p.y; o.normal=n; o.material=material; return o;
}
@fragment fn surface(i: GeometryOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    if g.settings.w > 1.5 { return vec4<f32>(vec3<f32>(i.height),1.0); }
    let roughness = array<f32,7>(0.88,0.66,0.28,0.18,0.24,0.92,0.57)[i.material];
    let metallic = select(0.0,1.0,i.material==2u || i.material==3u);
    let n=normalize(i.normal)*select(-1.0,1.0,front);
    let v=g.view.xyz; let l=g.light.xyz; let h=normalize(l+v);
    let nv=max(dot(n,v),0.0001); let nl=max(dot(n,l),0.0); let nh=max(dot(n,h),0.0);
    let a2=pow(roughness,4.0); let denom=nh*nh*(a2-1.0)+1.0;
    let d=a2/(3.14159265*denom*denom);
    let k=(roughness+1.0)*(roughness+1.0)/8.0;
    let visibility=(nv/(nv*(1.0-k)+k))*(nl/(nl*(1.0-k)+k));
    let f0=mix(vec3<f32>(0.04),i.color,metallic);
    let f=f0+(vec3<f32>(1.0)-f0)*pow(1.0-max(dot(h,v),0.0),5.0);
    let spec=d*visibility*f/max(4.0*nv*nl,0.0001);
    let diffuse=(vec3<f32>(1.0)-f)*(1.0-metallic)*i.color/3.14159265;
    return vec4<f32>(i.color*0.03+(diffuse+spec)*nl*3.14159265,1.0);
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
