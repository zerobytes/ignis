struct FieldVertex { p: vec4<f32>, n: vec4<f32> };
struct FieldParams {metric:vec4<f32>,grid:vec4<u32>,color:vec4<f32>,levels:array<vec4<u32>,6>};
@group(1) @binding(0) var<storage,read> field_heights:array<f32>;
@group(1) @binding(1) var<storage,read> field_vertices:array<FieldVertex>;
@group(1) @binding(2) var<uniform> field:FieldParams;
fn field_hash(value:u32)->u32{var x=value;x^=x>>16u;x*=0x7feb352du;x^=x>>15u;x*=0x846ca68bu;return x^(x>>16u);}
fn field_surface(block:vec4<u32>,vertex:u32)->VertexOut{
    let level=field.levels[block.w];let side=field.grid.z;let count=side/level.z;
    let plant=vertex/level.y;let lx=plant%count*level.z;let lz=plant/count*level.z;
    let source=block.x*side*side+lz*side+lx;let h=field_hash(source);
    let jitter=vec2<f32>(f32(h&65535u),f32(h>>16u))/65535.0-vec2<f32>(0.5);
    let root=vec3<f32>(field.metric.x+(f32(block.y*side+lx)+0.5+jitter.x*0.9)*field.metric.z,field_heights[source],field.metric.y+(f32(block.z*side+lz)+0.5+jitter.y*0.9)*field.metric.z);
    let v=field_vertices[level.x+vertex%level.y];let angle=f32(field_hash(h)&65535u)/65535.0*6.2831853;let c=cos(angle);let s=sin(angle);
    // Wider sparse representatives approximate far coverage; explicitly experimental.
    let widen=f32(level.z);let local=vec3<f32>(v.p.x*widen,v.p.y,v.p.z*widen);
    let position=root+vec3<f32>(c*local.x-s*local.z,local.y,s*local.x+c*local.z);
    var out:VertexOut;out.clip=globals.view_projection*vec4<f32>(position,1.0);out.world=position;
    out.normal=normalize(vec3<f32>(c*v.n.x-s*v.n.z,v.n.y,s*v.n.x+c*v.n.z));
    out.color=field.color.xyz*(0.82+f32(h&255u)/255.0*0.30);out.rest=position;out.material=0u;out.material_projection_weights=abs(out.normal);if root.y < -900000.0 {out.clip=vec4<f32>(2.0,2.0,2.0,1.0);}return out;
}
@vertex fn vs_field(@location(0) block:vec4<u32>,@builtin(vertex_index) vertex:u32)->VertexOut{return field_surface(block,vertex);}
@vertex fn vs_field_shadow(@location(0) block:vec4<u32>,@builtin(vertex_index) vertex:u32)->ShadowOut{let v=field_surface(block,vertex);var out:ShadowOut;out.clip=globals.shadow_projection*vec4<f32>(v.world,1.0);out.rest=v.rest;out.material=0u;if v.world.y < -900000.0 {out.clip=vec4<f32>(2.0,2.0,2.0,1.0);}return out;}
