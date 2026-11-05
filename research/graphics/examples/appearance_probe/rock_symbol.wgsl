struct Globals { matrix:mat4x4<f32>, inverse:mat4x4<f32>, eye:vec4<f32>, light:vec4<f32>, screen:vec4<f32> };
struct Symbol { center_radius:vec4<f32>, normal_thickness:vec4<f32>, color_area:vec4<f32> };
@group(0) @binding(0) var<uniform> g:Globals;
@group(0) @binding(1) var<storage,read> symbols:array<Symbol>;
@group(1) @binding(0) var nearest:texture_depth_2d;
@group(1) @binding(1) var accumulation:texture_2d<f32>;
fn material(n0:vec3<f32>,c:vec3<f32>,p:vec3<f32>)->vec3<f32>{
 let n=normalize(n0);let v=normalize(g.eye.xyz-p);let l=g.light.xyz;let h=normalize(l+v);
 let roughness=0.88;let nv=max(dot(n,v),0.0001);let nl=max(dot(n,l),0.0);let nh=max(dot(n,h),0.0);
 let a2=pow(roughness,4.0);let denom=nh*nh*(a2-1.0)+1.0;let d=a2/(3.14159265*denom*denom);
 let k=(roughness+1.0)*(roughness+1.0)/8.0;let visibility=(nv/(nv*(1.0-k)+k))*(nl/(nl*(1.0-k)+k));
 let f=vec3<f32>(0.04)+vec3<f32>(0.96)*pow(1.0-max(dot(h,v),0.0),5.0);
 let spec=d*visibility*f/max(4.0*nv*nl,0.0001);let diffuse=(vec3<f32>(1.0)-f)*c/3.14159265;
 return c*0.03+(diffuse+spec)*nl*3.14159265;
}
struct Surface { @builtin(position) clip:vec4<f32>, @location(0) world:vec3<f32>, @location(1) normal:vec3<f32>, @location(2) color:vec3<f32> };
@vertex fn mesh(@location(0) p:vec3<f32>,@location(1) n:vec3<f32>,@location(2) c:vec3<f32>,@location(3) m:u32)->Surface {
 var o:Surface;o.clip=g.matrix*vec4<f32>(p,1.0);o.world=p;o.normal=n;o.color=c;return o;
}
@fragment fn shade_mesh(i:Surface,@builtin(front_facing) front:bool)->@location(0) vec4<f32>{return vec4<f32>(material(i.normal*select(-1.0,1.0,front),i.color,i.world),1.0);}
struct Splat { @builtin(position) clip:vec4<f32>, @location(0) world:vec3<f32>, @location(1) uv:vec2<f32>, @location(2) @interpolate(flat) id:u32 };
@vertex fn splat(@builtin(vertex_index) v:u32,@builtin(instance_index) id:u32)->Splat {
 let s=symbols[id];let n=s.normal_thickness.xyz;let a=select(vec3<f32>(0.0,1.0,0.0),vec3<f32>(1.0,0.0,0.0),abs(n.y)>0.9);
 let u=normalize(cross(a,n));let w=cross(n,u);
 let corners=array<vec2<f32>,6>(vec2<f32>(-1.0,-1.0),vec2<f32>(1.0,-1.0),vec2<f32>(-1.0,1.0),vec2<f32>(-1.0,1.0),vec2<f32>(1.0,-1.0),vec2<f32>(1.0,1.0));
 let uv=corners[v];let p=s.center_radius.xyz+(u*uv.x+w*uv.y)*s.center_radius.w;
 var o:Splat;o.clip=g.matrix*vec4<f32>(p,1.0);o.world=p;o.uv=uv;o.id=id;return o;
}
fn rejected(i:Splat)->bool {let s=symbols[i.id];return dot(i.uv,i.uv)>1.0 || dot(s.normal_thickness.xyz,g.eye.xyz-s.center_radius.xyz)<=0.0;}
@fragment fn depth_splat(i:Splat){if rejected(i){discard;}}
@fragment fn shade_splat(i:Splat)->@location(0) vec4<f32>{if rejected(i){discard;}let s=symbols[i.id];return vec4<f32>(material(s.normal_thickness.xyz,s.color_area.xyz,i.world),1.0);}
fn world_at(pixel:vec2<f32>,depth:f32)->vec3<f32>{let clip=vec4<f32>(pixel/g.screen.xy*vec2<f32>(2.0,-2.0)+vec2<f32>(-1.0,1.0),depth,1.0);let p=g.inverse*clip;return p.xyz/p.w;}
@fragment fn accumulate(i:Splat)->@location(0) vec4<f32>{
 if rejected(i){discard;}let s=symbols[i.id];let d=textureLoad(nearest,vec2<i32>(i.clip.xy),0);if d==0.0{discard;}
 let p=world_at(i.clip.xy,d);let separation=length(g.eye.xyz-i.world)-length(g.eye.xyz-p);
 // Explicit approximation: allow neighbours only within source patch thickness.
 let tolerance=max(2.0*s.normal_thickness.w,0.02*s.center_radius.w);
 if separation>tolerance{discard;}
 let weight=max(0.001,1.0-dot(i.uv,i.uv));let color=material(s.normal_thickness.xyz,s.color_area.xyz,i.world);
 return vec4<f32>(color*weight,weight);
}
@vertex fn screen(@builtin(vertex_index) id:u32)->@builtin(position)vec4<f32>{let p=array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0));return vec4<f32>(p[id],0.0,1.0);}
@fragment fn resolve(@builtin(position) p:vec4<f32>)->@location(0)vec4<f32>{let c=textureLoad(accumulation,vec2<i32>(p.xy),0);if c.a<=0.0{return vec4<f32>(0.04,0.07,0.11,0.0);}return vec4<f32>(c.rgb/c.a,1.0);}
