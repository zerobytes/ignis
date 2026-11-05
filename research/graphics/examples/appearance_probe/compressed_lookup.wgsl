struct Region { origin: vec4<u32>, extent: vec4<u32> };
@group(0) @binding(3) var<storage,read> regions: array<Region>;
fn lookup(uv: vec2<f32>, v: i32, l: f32) -> vec4<f32> {
    if any(uv<vec2<f32>(0.0)) || any(uv>vec2<f32>(1.0)) {return vec4<f32>(0.0);}
    let q=clamp(uv,vec2<f32>(0.5/128.0),vec2<f32>(1.0-0.5/128.0));
    let p=q*128.0;
    let band=min(u32(p.y)/16u,7u);
    let r=regions[(u32(v)*25u+u32(l))*8u+band];
    let local=p-vec2<f32>(r.origin.zw);
    if r.extent.x==0u || any(local<vec2<f32>(0.5)) || any(local>vec2<f32>(r.extent.xy)-vec2<f32>(0.5)) {return vec4<f32>(0.0);}
    let packed=(vec2<f32>(r.origin.xy)+local)/vec2<f32>(textureDimensions(atlas));
    return textureSampleLevel(atlas,filtering,packed,0,0.0);
}
