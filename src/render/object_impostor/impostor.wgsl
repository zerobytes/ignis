struct ObjectAtlasJob {world:mat4x4<f32>,eye:vec4<f32>,bounds:vec4<f32>,control:vec4<u32>,mix_lod:vec4<f32>};
struct ObjectPalette {ratio:vec3<f32>,material:u32};
struct ObjectAtlasParameters {grid:vec4<u32>,pixel:vec4<f32>,elevations:vec4<f32>};
@group(1) @binding(0) var object_base:texture_2d_array<f32>;
@group(1) @binding(1) var object_normal:texture_2d_array<f32>;
@group(1) @binding(2) var object_depth:texture_2d_array<u32>;
@group(1) @binding(3) var object_sampler:sampler;
@group(1) @binding(4) var<storage,read> object_jobs:array<ObjectAtlasJob>;
@group(1) @binding(5) var<storage,read> object_palette:array<ObjectPalette>;
@group(1) @binding(6) var<uniform> object_parameters:ObjectAtlasParameters;
struct ObjectAtlasOut {@builtin(position) clip:vec4<f32>,@location(0) @interpolate(flat) instance:u32};
@vertex fn vs_object_atlas(@builtin(vertex_index) vertex:u32,@builtin(instance_index) instance:u32)->ObjectAtlasOut {
    let q=array<vec2<f32>,6>(vec2<f32>(-1.0,-1.0),vec2<f32>(1.0,-1.0),vec2<f32>(-1.0,1.0),vec2<f32>(-1.0,1.0),vec2<f32>(1.0,-1.0),vec2<f32>(1.0,1.0))[vertex];
    let j=object_jobs[instance];let center=(j.world*vec4<f32>(j.bounds.xyz,1.0)).xyz;let radius=j.bounds.w*length(j.world[0].xyz)*1.01;
    let p=center-globals.atmosphere_forward.xyz*radius+(normalize(globals.atmosphere_right.xyz)*q.x+normalize(globals.atmosphere_up.xyz)*q.y)*radius;
    var out:ObjectAtlasOut;out.clip=globals.view_projection*vec4<f32>(p,1.0);out.instance=instance;return out;
}
struct ObjectHit {position:vec3<f32>,normal:vec3<f32>,color:vec3<f32>,alpha:f32,material:u32};
fn object_trace(j:ObjectAtlasJob,origin:vec3<f32>,direction:vec3<f32>,layer:u32)->ObjectHit {
    var out:ObjectHit;out.position=(j.world*vec4<f32>(j.bounds.xyz,1.0)).xyz;out.normal=vec3<f32>(0.0,1.0,0.0);out.color=vec3<f32>(0.0);out.alpha=0.0;out.material=0u;
    let az=f32(layer%object_parameters.grid.y)*6.28318530718/f32(object_parameters.grid.y);
    let elevation=object_parameters.elevations[(layer/object_parameters.grid.y)%object_parameters.grid.z];
    let forward=-vec3<f32>(sin(az)*cos(elevation),sin(elevation),cos(az)*cos(elevation));let right=normalize(cross(forward,vec3<f32>(0.0,1.0,0.0)));let up=cross(right,forward);
    let denom=dot(direction,forward);if abs(denom)<0.01{return out;}
    var t=dot(j.bounds.xyz-origin,forward)/denom;var uv=vec2<f32>(0.5);let mip=i32(clamp(floor(j.mix_lod.y),0.0,f32(object_parameters.grid.w-1u)));let side=i32(object_parameters.grid.x>>u32(mip));
    for(var step=0u;step<4u;step++){
        let p=origin+t*direction-j.bounds.xyz;uv=vec2<f32>(0.5+0.5*dot(p,right)/j.bounds.w,0.5-0.5*dot(p,up)/j.bounds.w);
        if any(uv<vec2<f32>(0.0))||any(uv>vec2<f32>(1.0)){return out;}
        let xy=clamp(vec2<i32>(uv*f32(side)),vec2<i32>(0),vec2<i32>(side-1));
        let z=(f32(textureLoad(object_depth,xy,i32(layer),mip).r)/65535.0*2.0-1.0)*j.bounds.w;
        t=(z-dot(origin-j.bounds.xyz,forward))/denom;
    }
    let p=origin+t*direction-j.bounds.xyz;uv=vec2<f32>(0.5+0.5*dot(p,right)/j.bounds.w,0.5-0.5*dot(p,up)/j.bounds.w);
    if t<=0.0||any(uv<vec2<f32>(0.0))||any(uv>vec2<f32>(1.0)){return out;}
    let base=textureSampleLevel(object_base,object_sampler,uv,i32(layer),j.mix_lod.y);let normal=(textureSampleLevel(object_normal,object_sampler,uv,i32(layer),j.mix_lod.y).xyz*255.0-128.0)/127.0;
    let xy=clamp(vec2<i32>(uv*f32(side)),vec2<i32>(0),vec2<i32>(side-1));let part=u32(round(textureLoad(object_normal,xy,i32(layer),mip).a*255.0));let palette=object_palette[j.control.x+part];
    out.position=(j.world*vec4<f32>(origin+t*direction,1.0)).xyz;out.normal=normalize((j.world*vec4<f32>(select(vec3<f32>(0.0,1.0,0.0),normal,dot(normal,normal)>0.00000001),0.0)).xyz);out.color=base.xyz/max(base.a,0.00001)*palette.ratio;out.alpha=base.a;out.material=palette.material;return out;
}
fn object_vertex(h:ObjectHit)->VertexOut {var v:VertexOut;v.clip=globals.view_projection*vec4<f32>(h.position,1.0);v.world=h.position;v.normal=h.normal;v.color=h.color;v.rest=vec3<f32>(0.0);v.material=h.material;v.material_projection_weights=vec3<f32>(0.0);return v;}
struct ObjectAtlasFragment {@location(0) color:vec4<f32>,@builtin(frag_depth) depth:f32};
@fragment fn fs_object_atlas(in:ObjectAtlasOut)->ObjectAtlasFragment {
    let j=object_jobs[in.instance];let ndc=in.clip.xy*object_parameters.pixel.xy+object_parameters.pixel.zw;
    let ray=normalize(globals.atmosphere_forward.xyz+ndc.x*globals.atmosphere_right.xyz+ndc.y*globals.atmosphere_up.xyz);
    let direction=normalize(vec3<f32>(dot(j.world[0].xyz,ray),dot(j.world[1].xyz,ray),dot(j.world[2].xyz,ray)));
    let a=object_trace(j,j.eye.xyz,direction,j.control.y);let b=object_trace(j,j.eye.xyz,direction,j.control.z);
    var ca=shade_surface(object_vertex(a),true).color;var cb=shade_surface(object_vertex(b),true).color;
    let wa=a.alpha*(1.0-j.mix_lod.x);let wb=b.alpha*j.mix_lod.x;let alpha=wa+wb;
    if alpha<0.0039215686 {discard;}
    let world=(a.position*wa+b.position*wb)/alpha;let clip=globals.view_projection*vec4<f32>(world,1.0);let depth=clip.z/clip.w;
    if depth<0.0||depth>1.0 {discard;}
    var out:ObjectAtlasFragment;out.color=vec4<f32>(ca*wa+cb*wb,alpha);out.depth=depth;return out;
}

struct ObjectShadowOut {@builtin(position) clip:vec4<f32>,@location(0) @interpolate(flat) instance:u32,@location(1) plane:vec3<f32>};
@vertex fn vs_object_shadow(@builtin(vertex_index) vertex:u32,@builtin(instance_index) instance:u32)->ObjectShadowOut {
    let q=array<vec2<f32>,6>(vec2<f32>(-1.0,-1.0),vec2<f32>(1.0,-1.0),vec2<f32>(-1.0,1.0),vec2<f32>(-1.0,1.0),vec2<f32>(1.0,-1.0),vec2<f32>(1.0,1.0))[vertex];
    let j=object_jobs[instance];let center=(j.world*vec4<f32>(j.bounds.xyz,1.0)).xyz;let radius=j.bounds.w*length(j.world[0].xyz)*1.01;
    let direction=normalize(globals.light.xyz);let reference=select(vec3<f32>(0.0,1.0,0.0),vec3<f32>(0.0,0.0,1.0),abs(direction.y)>0.98);let right=normalize(cross(direction,reference));let up=cross(right,direction);
    let p=center-direction*radius+(right*q.x+up*q.y)*radius;var out:ObjectShadowOut;out.clip=globals.shadow_projection*vec4<f32>(p,1.0);out.plane=p;out.instance=instance;return out;
}
fn object_noise(value:u32)->u32 {var x=value;x^=x>>16u;x*=0x7feb352du;x^=x>>15u;x*=0x846ca68bu;return x^(x>>16u);}
@fragment fn fs_object_shadow(in:ObjectShadowOut)->@builtin(frag_depth) f32 {
    let j=object_jobs[in.instance];let scale2=dot(j.world[0].xyz,j.world[0].xyz);let p=in.plane-j.world[3].xyz;
    let origin=vec3<f32>(dot(j.world[0].xyz,p),dot(j.world[1].xyz,p),dot(j.world[2].xyz,p))/scale2;
    let ray=normalize(globals.light.xyz);let direction=normalize(vec3<f32>(dot(j.world[0].xyz,ray),dot(j.world[1].xyz,ray),dot(j.world[2].xyz,ray)));
    let a=object_trace(j,origin,direction,j.control.y);let b=object_trace(j,origin,direction,j.control.z);let wa=a.alpha*(1.0-j.mix_lod.x);let wb=b.alpha*j.mix_lod.x;let alpha=wa+wb;
    let seed=u32(in.clip.x)+u32(in.clip.y)*65537u+j.control.w*0x9e3779b9u;let choice=f32(object_noise(seed)&65535u)/65536.0;
    if alpha<=choice||alpha<0.0039215686 {discard;}
    let world=(a.position*wa+b.position*wb)/alpha;let clip=globals.shadow_projection*vec4<f32>(world,1.0);let depth=clip.z/clip.w;if depth<0.0||depth>1.0{discard;}return depth;
}
