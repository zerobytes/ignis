@group(1) @binding(0) var<uniform> object_bake:vec4<f32>;
struct ObjectBakeOut {
    @location(0) base:vec4<f32>,
    @location(1) normal:vec4<f32>,
    @location(2) depth:f32,
    @location(3) part:u32,
};
@fragment fn fs_object_bake(in:VertexOut,@builtin(front_facing) front:bool)->ObjectBakeOut {
    let s=shade_surface(in,front);
    var out:ObjectBakeOut;
    out.base=vec4<f32>(s.base,1.0);out.normal=vec4<f32>(s.normal,0.0);
    out.depth=0.5+0.5*dot(in.world-object_bake.xyz,globals.atmosphere_forward.xyz)/object_bake.w;
    out.part=in.material;return out;
}
