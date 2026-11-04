struct FieldVertex { p: vec4<f32>, n: vec4<f32> };
struct FieldParams {metric:vec4<f32>,grid:vec4<u32>,color:vec4<f32>,levels:array<vec4<u32>,6>,profile:array<vec4<f32>,2>};
struct BladeVertex {root:vec4<f32>,center:vec4<f32>};
@group(1) @binding(0) var<storage,read> field_heights:array<f32>;
@group(1) @binding(1) var<storage,read> field_vertices:array<FieldVertex>;
@group(1) @binding(2) var<uniform> field:FieldParams;
@group(1) @binding(3) var<storage,read> blade_vertices:array<BladeVertex>;
@group(1) @binding(4) var<storage,read> planting_mask:array<u32>;
fn field_hash(value:u32)->u32{var x=value;x^=x>>16u;x*=0x7feb352du;x^=x>>15u;x*=0x846ca68bu;return x^(x>>16u);}
fn terrain_at(p:vec2<u32>)->f32{
    let side=field.grid.z;
    return field_heights[((p.y/side)*field.grid.x+p.x/side)*side*side+(p.y%side)*side+p.x%side];
}
fn terrain_height(xz:vec2<f32>)->f32{
    let dimensions=field.grid.xy*field.grid.z;
    let q=(xz-field.metric.xy)/field.metric.z-vec2<f32>(0.5);
    let b=vec2<u32>(clamp(floor(q),vec2<f32>(0.0),vec2<f32>(dimensions-vec2<u32>(2u))));
    let t=clamp(q-vec2<f32>(b),vec2<f32>(0.0),vec2<f32>(1.0));
    let a=terrain_at(b);let c=terrain_at(b+vec2<u32>(1u,0u));let d=terrain_at(b+vec2<u32>(0u,1u));let e=terrain_at(b+vec2<u32>(1u));
    if t.x+t.y<=1.0 {return a+(c-a)*t.x+(d-a)*t.y;}
    return e+(d-e)*(1.0-t.x)+(c-e)*(1.0-t.y);
}
fn threshold(i:u32)->f32{
    if i<4u {return field.profile[0][i];}return field.profile[1].x;
}
fn field_surface(block:vec4<u32>,vertex:u32,shadow:bool)->VertexOut{
    let level=field.levels[block.w];let side=field.grid.z;let count=side/level.z;
    let plant=vertex/level.y;let lx=plant%count*level.z;let lz=plant/count*level.z;
    let source=block.x*side*side+lz*side+lx;let h=field_hash(source);
    let jitter=vec2<f32>(f32(h&65535u),f32(h>>16u))/65535.0-vec2<f32>(0.5);
    let root=vec3<f32>(field.metric.x+(f32(block.y*side+lx)+0.5+jitter.x*0.9)*field.metric.z,field_heights[source],field.metric.y+(f32(block.z*side+lz)+0.5+jitter.y*0.9)*field.metric.z);
    let v=field_vertices[level.x+vertex%level.y];let angle=f32(field_hash(h)&65535u)/65535.0*6.2831853;let c=cos(angle);let s=sin(angle);
    // Wider sparse representatives approximate far coverage; explicitly experimental.
    let widen=f32(level.z);let local=vec3<f32>(v.p.x*widen,v.p.y,v.p.z*widen);
    var position=root+vec3<f32>(c*local.x-s*local.z,local.y,s*local.x+c*local.z);
    var normal=normalize(vec3<f32>(c*v.n.x-s*v.n.z,v.n.y,s*v.n.x+c*v.n.z));
    var color=field.color.xyz*(0.82+f32(h&255u)/255.0*0.30);
    var hidden=root.y < -900000.0;
    var excluded=false;
    if field.grid.w>0u {
        let aux=blade_vertices[level.x+vertex%level.y];
        let seed=field_hash(source^(u32(aux.root.w)*0x9e3779b9u));
        let h2=field_hash(seed^0x85ebca6bu);
        let uv=vec2<f32>(f32(seed&65535u),f32(seed>>16u))/65536.0;
        let xz=field.metric.xy+(vec2<f32>(vec2<u32>(block.y*side+lx,block.z*side+lz))+uv*f32(level.z))*field.metric.z;
        let base=vec3<f32>(xz.x,terrain_height(xz)-0.002,xz.y);
        if (field.grid.w&2u)!=0u {
            let cell=vec2<u32>(clamp(floor((xz-field.metric.xy)/field.metric.z),vec2<f32>(0.0),vec2<f32>(field.grid.xy*side-vec2<u32>(1u))));
            let k=((cell.y/side)*field.grid.x+cell.x/side)*side*side+(cell.y%side)*side+cell.x%side;
            excluded=(planting_mask[k/32u]&(1u<<(k%32u)))==0u;
        }
        let metric=distance(base,globals.view_position.xyz)/max(field.metric.w,1.0);
        let choice=f32(h2&65535u)/65536.0;
        var selected=0u;
        for(var i=0u;i<5u;i+=1u){
            let t=threshold(i);let blend=field.profile[1].y;
            if choice<smoothstep(t*(1.0-blend),t*(1.0+blend),metric) {selected=i+1u;}
        }
        hidden=!shadow && selected!=block.w;
        let ideal=field.color.w*f32(level.z*level.z)/f32(max(level.w,1u));
        let width=min(8.0,min(ideal,max(1.0,0.8*metric/field.profile[1].z)));
        let p=(aux.center.xyz-aux.root.xyz)+(v.p.xyz-aux.center.xyz)*width;
        let angle2=f32(h2>>16u)/65536.0*6.2831853;let c2=cos(angle2);let s2=sin(angle2);
        position=base+vec3<f32>(c2*p.x-s2*p.z,p.y,s2*p.x+c2*p.z);
        normal=normalize(vec3<f32>(c2*v.n.x-s2*v.n.z,v.n.y,s2*v.n.x+c2*v.n.z));
        let height=clamp((v.p.y-aux.root.y)/field.profile[1].w,0.0,1.0);
        color=field.color.xyz*aux.center.w*(0.94+f32(seed&255u)/255.0*0.12)*mix(0.68,1.06,sqrt(height));
    }
    if excluded {position.y=-1000000.0;hidden=true;}
    var out:VertexOut;out.clip=globals.view_projection*vec4<f32>(position,1.0);out.world=position;
    out.normal=normal;
    out.color=color;out.rest=position;out.material=0u;out.material_projection_weights=abs(out.normal);if hidden {out.clip=vec4<f32>(2.0,2.0,2.0,1.0);}return out;
}
@vertex fn vs_field(@location(0) block:vec4<u32>,@builtin(vertex_index) vertex:u32)->VertexOut{return field_surface(block,vertex,false);}
@vertex fn vs_field_shadow(@location(0) block:vec4<u32>,@builtin(vertex_index) vertex:u32)->ShadowOut{let v=field_surface(block,vertex,true);var out:ShadowOut;out.clip=globals.shadow_projection*vec4<f32>(v.world,1.0);out.rest=v.rest;out.material=0u;if v.world.y < -900000.0 {out.clip=vec4<f32>(2.0,2.0,2.0,1.0);}return out;}
@fragment fn fs_field(in:VertexOut,@builtin(front_facing)front:bool)->@location(0)vec4<f32>{
    var v=in;if !front {v.normal=-v.normal;}
    return vec4<f32>(shade_surface(v,true).color,1.0);
}
