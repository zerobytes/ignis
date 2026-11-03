struct CollectiveInstance { world:mat4x4<f32>, control:vec4<u32>, };
struct CollectiveColor { color:vec3<f32>, padding:u32, };
struct CollectiveParameters { bounds:vec4<f32>, pixel_to_ndc:vec4<f32>, };
@group(1) @binding(0) var<storage,read> collective_triangles:array<u32>;
@group(1) @binding(1) var<storage,read> collective_nodes:array<u32>;
@group(1) @binding(2) var<uniform> collective_parameters:CollectiveParameters;
@group(1) @binding(3) var<storage,read> collective_instances:array<CollectiveInstance>;
@group(1) @binding(4) var<storage,read> collective_colors:array<CollectiveColor>;
@group(1) @binding(5) var collective_seed:texture_depth_2d;
@group(1) @binding(6) var<uniform> collective_seed_settings:vec4<f32>;
struct CollectiveOut { @builtin(position) clip:vec4<f32>,@location(0) @interpolate(flat) instance:u32, };
fn collective_local_vector(i:CollectiveInstance,p:vec3<f32>)->vec3<f32>{
    let scale2=dot(i.world[0].xyz,i.world[0].xyz);
    return vec3<f32>(dot(i.world[0].xyz,p),dot(i.world[1].xyz,p),dot(i.world[2].xyz,p))/scale2;
}
@vertex fn vs_collective(@builtin(vertex_index) vertex:u32,@builtin(instance_index) instance:u32)->CollectiveOut {
    let q=array<vec2<f32>,6>(vec2<f32>(-1.0,-1.0),vec2<f32>(1.0,-1.0),vec2<f32>(-1.0,1.0),vec2<f32>(-1.0,1.0),vec2<f32>(1.0,-1.0),vec2<f32>(1.0,1.0))[vertex];
    let i=collective_instances[instance];let bounds=collective_parameters.bounds;
    let eye=collective_local_vector(i,globals.view_position.xyz-i.world[3].xyz);
    let direction=normalize(eye-bounds.xyz);let reference=select(vec3<f32>(0.0,1.0,0.0),vec3<f32>(0.0,0.0,1.0),abs(direction.y)>0.98);
    let right=normalize(cross(reference,direction));let up=cross(direction,right);
    let pad=collective_pad(i,eye,bounds.xyz-vec3<f32>(bounds.w),bounds.xyz+vec3<f32>(bounds.w));
    let local=bounds.xyz+(bounds.w+pad)*(right*q.x+up*q.y+direction);
    var out:CollectiveOut;out.clip=globals.view_projection*i.world*vec4<f32>(local,1.0);out.instance=instance;return out;
}
fn collective_node_vec(at:u32)->vec3<f32>{return bitcast<vec3<f32>>(vec3<u32>(collective_nodes[at],collective_nodes[at+1u],collective_nodes[at+2u]));}
fn collective_triangle_vec(at:u32)->vec3<f32>{return bitcast<vec3<f32>>(vec3<u32>(collective_triangles[at],collective_triangles[at+1u],collective_triangles[at+2u]));}
fn collective_box(origin:vec3<f32>,direction:vec3<f32>,lo:vec3<f32>,hi:vec3<f32>,limit:f32)->bool {
    var near=0.0;var far=limit;
    for(var axis=0u;axis<3u;axis++){
        if direction[axis]==0.0 {if origin[axis]<lo[axis]||origin[axis]>hi[axis]{return false;}}
        else{let a=(lo[axis]-origin[axis])/direction[axis];let b=(hi[axis]-origin[axis])/direction[axis];near=max(near,min(a,b));far=min(far,max(a,b));}
    }
    return far>=near;
}
struct CollectiveFragment { @location(0) color:vec4<f32>,@builtin(frag_depth) depth:f32, };
@fragment fn fs_collective(in:CollectiveOut)->CollectiveFragment {
    let i=collective_instances[in.instance];let ndc=in.clip.xy*collective_parameters.pixel_to_ndc.xy+collective_parameters.pixel_to_ndc.zw;
    let world_direction=normalize(globals.atmosphere_forward.xyz+ndc.x*globals.atmosphere_right.xyz+ndc.y*globals.atmosphere_up.xyz);
    let direction=normalize(collective_local_vector(i,world_direction));let origin=collective_local_vector(i,globals.view_position.xyz-i.world[3].xyz);
    if collective_seed_settings.x>0.0 {
        let root_lo=collective_node_vec(0u);let root_hi=collective_node_vec(4u);
        let root_pad=vec3<f32>(collective_pad(i,origin,root_lo,root_hi));
        let lo=root_lo-root_pad;let hi=root_hi+root_pad;
        var entry=0.0;var exit=1e30;
        for(var axis=0u;axis<3u;axis++) {
            if direction[axis]==0.0 {if origin[axis]<lo[axis]||origin[axis]>hi[axis]{discard;}}
            else {let a=(lo[axis]-origin[axis])/direction[axis];let b=(hi[axis]-origin[axis])/direction[axis];entry=max(entry,min(a,b));exit=min(exit,max(a,b));}
        }
        if exit<entry {discard;}
        let entry_clip=globals.view_projection*i.world*vec4<f32>(origin+entry*direction,1.0);
        let stored=textureLoad(collective_seed,vec2<i32>(in.clip.xy),0);
        if entry_clip.w>0.0 && stored>0.0 {
            let candidate=collective_seed_settings.y/entry_clip.w;
            let a=30000.0/(30000.0-0.05);let scale=0.05*a/collective_seed_settings.y;
            // Match the native culler's conservative conventional-depth tie guard.
            if candidate*collective_seed_settings.z<stored && (a-scale*candidate)>(a-scale*stored)+collective_seed_settings.w {discard;}
        }
    }
    var node=0u;var closest=1e30;var closest_depth=1.0;var closest_order=1e30;var face=0xffffffffu;var hit=0xffffffffu;var bary=vec2<f32>(0.0);
    let node_count=arrayLength(&collective_nodes)/8u;
    loop {
        if node>=node_count {break;}
        let at=node*8u;let escape=collective_nodes[at+3u];let descriptor=collective_nodes[at+7u];
        let lo=collective_node_vec(at);let hi=collective_node_vec(at+4u);let pad=vec3<f32>(collective_pad(i,origin,lo,hi));
        if !collective_box(origin,direction,lo-pad,hi+pad,select(closest,1e30,i.control.y!=0u && i.control.z!=0u)){node=escape;continue;}
        let count=descriptor&15u;
        if count==0u {node+=1u;continue;}
        let first=descriptor>>4u;
        for(var leaf=0u;leaf<count;leaf++){
            let tri=(first+leaf)*20u;
            if i.control.y!=0u {
                let sample=collective_raster(i,tri,in.clip.xy);let id=collective_triangles[tri+19u];
                let order=select(sample.w,sample.z,i.control.z!=0u);
                if sample.w>0.0 && sample.z>=0.0 && sample.z<=1.0 && (order<closest_order || (order==closest_order && id<face)) {
                    closest_order=order;closest_depth=sample.z;face=id;hit=tri;bary=sample.xy;
                    if i.control.z==0u {closest=sample.w/(length(i.world[0].xyz)*dot(world_direction,globals.atmosphere_forward.xyz));}
                }
                continue;
            }
            let a=collective_triangle_vec(tri);let e1=collective_triangle_vec(tri+3u)-a;let e2=collective_triangle_vec(tri+6u)-a;
            let p=cross(direction,e2);let determinant=dot(e1,p);if determinant==0.0{continue;}
            let inverse=1.0/determinant;let s=origin-a;let u=dot(s,p)*inverse;if u<0.0||u>1.0 {continue;}
            let q=cross(s,e1);let v=dot(direction,q)*inverse;if v<0.0||u+v>1.0 {continue;}
            let t=dot(e2,q)*inverse;let id=collective_triangles[tri+19u];
            if t>=0.0&&(t<closest||(t==closest&&id<face)){closest=t;face=id;hit=tri;bary=vec2<f32>(u,v);}
        }
        node=escape;
    }
    if hit==0xffffffffu{discard;}
    var local=origin+closest*direction;
    if i.control.y!=0u {
        local=collective_triangle_vec(hit)*(1.0-bary.x-bary.y)+collective_triangle_vec(hit+3u)*bary.x+collective_triangle_vec(hit+6u)*bary.y;
    }let world=(i.world*vec4<f32>(local,1.0)).xyz;let clip=globals.view_projection*vec4<f32>(world,1.0);
    let depth=select(clip.z/clip.w,closest_depth,i.control.y!=0u);if depth<0.0||depth>1.0{discard;}
    let n=collective_triangle_vec(hit+9u)*(1.0-bary.x-bary.y)+collective_triangle_vec(hit+12u)*bary.x+collective_triangle_vec(hit+15u)*bary.y;
    let normal=normalize((i.world*vec4<f32>(n,0.0)).xyz);let base=collective_colors[i.control.x+collective_triangles[hit+18u]].color;
    let diffuse=max(dot(normal,-normalize(globals.light.xyz)),0.0);var color=base*(globals.light.w+(1.0-globals.light.w)*diffuse);
    if globals.environment.z!=1.0 {let ambient=base*globals.light.w;color=ambient+(color-ambient)*globals.environment.z;}
    if globals.atmosphere.w>0.0||globals.atmosphere_sky.w>0.0 {color=atmosphere_surface(color,world);}
    var out:CollectiveFragment;out.color=vec4<f32>(color,1.0);out.depth=depth;return out;
}
