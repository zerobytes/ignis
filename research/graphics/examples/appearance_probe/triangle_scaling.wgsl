struct Uniforms {matrix:mat4x4<f32>,eye:vec4<f32>,light:vec4<f32>,settings:vec4<u32>};
@group(0) @binding(0) var<uniform> g:Uniforms;
@group(0) @binding(1) var normals:texture_2d<f32>;
struct Surface {@builtin(position)clip:vec4<f32>,@location(0)world:vec3<f32>,@location(1)normal:vec3<f32>};
@vertex fn vertex(@location(0)p:vec3<f32>,@location(1)n:vec3<f32>)->Surface {var s:Surface;s.clip=g.matrix*vec4<f32>(p,1);s.world=p;s.normal=n;return s;}
fn source_normal(world:vec3<f32>)->vec3<f32>{
 let dims=vec2<i32>(textureDimensions(normals))-1;let p=clamp((world.xz/12.0+0.5)*vec2<f32>(dims),vec2<f32>(0),vec2<f32>(dims));let a=min(vec2<i32>(floor(p)),dims-1);let f=p-vec2<f32>(a);
 if f.x+f.y<=1.0 {return textureLoad(normals,a,0).xyz*(1.0-f.x-f.y)+textureLoad(normals,a+vec2<i32>(1,0),0).xyz*f.x+textureLoad(normals,a+vec2<i32>(0,1),0).xyz*f.y;}
 return textureLoad(normals,a+vec2<i32>(1,1),0).xyz*(f.x+f.y-1.0)+textureLoad(normals,a+vec2<i32>(0,1),0).xyz*(1.0-f.x)+textureLoad(normals,a+vec2<i32>(1,0),0).xyz*(1.0-f.y);
}
@fragment fn fragment(s:Surface)->@location(0)vec4<f32>{
 var n=s.normal;if g.settings.x==3u {n=source_normal(s.world);}n=normalize(n);
 let l=normalize(g.light.xyz);let v=normalize(g.eye.xyz-s.world);let h=normalize(l+v);let nl=max(dot(n,l),0.0);
 let color=vec3<f32>(0.36,0.29,0.19);let spec=0.08*pow(max(dot(n,h),0.0),24.0);
 return vec4<f32>(color*(0.15+0.85*nl)+vec3<f32>(spec),1.0);
}

// Diagnostic entry points share geometry/depth/target. Not quality modes.
@fragment fn flat_fragment(s:Surface)->@location(0)vec4<f32>{return vec4<f32>(0.36,0.29,0.19,1.0);}
@fragment fn normal_fragment(s:Surface)->@location(0)vec4<f32>{
 var n=s.normal;if g.settings.x==3u {n=source_normal(s.world);}
 return vec4<f32>(normalize(n)*0.5+0.5,1.0);
}
fn material(n0:vec3<f32>,world:vec3<f32>)->vec4<f32>{
 let n=normalize(n0);let l=normalize(g.light.xyz);let v=normalize(g.eye.xyz-world);let h=normalize(l+v);let nl=max(dot(n,l),0.0);
 let color=vec3<f32>(0.36,0.29,0.19);let spec=0.08*pow(max(dot(n,h),0.0),24.0);
 return vec4<f32>(color*(0.15+0.85*nl)+vec3<f32>(spec),1.0);
}
@fragment fn native_fragment(s:Surface)->@location(0)vec4<f32>{return material(s.normal,s.world);}
fn branchless_normal(world:vec3<f32>)->vec3<f32>{
 let dims=vec2<i32>(textureDimensions(normals))-1;let p=clamp((world.xz/12.0+0.5)*vec2<f32>(dims),vec2<f32>(0),vec2<f32>(dims));let a=min(vec2<i32>(floor(p)),dims-1);let f=p-vec2<f32>(a);
 let lower=f.x+f.y<=1.0;
 let anchor=select(a+vec2<i32>(1,1),a,lower);
 let second=select(a+vec2<i32>(0,1),a+vec2<i32>(1,0),lower);
 let third=select(a+vec2<i32>(1,0),a+vec2<i32>(0,1),lower);
 let weights=select(vec3<f32>(f.x+f.y-1.0,1.0-f.x,1.0-f.y),vec3<f32>(1.0-f.x-f.y,f.x,f.y),lower);
 return textureLoad(normals,anchor,0).xyz*weights.x+textureLoad(normals,second,0).xyz*weights.y+textureLoad(normals,third,0).xyz*weights.z;
}
@fragment fn branchless_fragment(s:Surface)->@location(0)vec4<f32>{
 var n=s.normal;if g.settings.x==3u {n=branchless_normal(s.world);}return material(n,s.world);
}

// Signed reverse depth records the source of the normal. No material is evaluated here.
@fragment fn gbuffer_fragment(s:Surface)->@location(0)vec4<f32>{
 if g.settings.x==3u {return vec4<f32>(0.0,0.0,0.0,-s.clip.z);}
 return vec4<f32>(s.normal,s.clip.z);
}
@fragment fn native_gbuffer_fragment(s:Surface)->@location(0)vec4<f32>{return vec4<f32>(s.normal,s.clip.z);}

@vertex fn position_vertex(@location(0)p:vec3<f32>)->@builtin(position)vec4<f32>{return g.matrix*vec4<f32>(p,1);}
