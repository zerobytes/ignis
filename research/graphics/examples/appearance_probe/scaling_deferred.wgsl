
struct Reconstruct { inverse:mat4x4<f32> };
@group(1) @binding(0) var surface:texture_2d<f32>;
@group(1) @binding(1) var<uniform> reconstruction:Reconstruct;
@vertex fn fullscreen(@builtin(vertex_index) i:u32)->@builtin(position)vec4<f32>{
 let p=array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0));return vec4<f32>(p[i],0.0,1.0);
}
@fragment fn shade_visible(@builtin(position) p:vec4<f32>)->@location(0)vec4<f32>{
 let entry=textureLoad(surface,vec2<i32>(p.xy),0);
 if entry.w==0.0 {return vec4<f32>(0.04,0.07,0.11,0.0);}
 let size=vec2<f32>(textureDimensions(surface));
 let clip=vec4<f32>(2.0*p.x/size.x-1.0,1.0-2.0*p.y/size.y,abs(entry.w),1.0);
 let homogeneous=reconstruction.inverse*clip;let world=homogeneous.xyz/homogeneous.w;
 var n=entry.xyz;if entry.w<0.0 {n=branchless_normal(world);}
 return material(n,world);
}
