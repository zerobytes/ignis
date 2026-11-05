// Diagnostic only: same RGBA8 mip chain, explicit clamp-to-edge and trilinear.
fn texel_bilinear(map:texture_2d<f32>, uv:vec2<f32>, level:i32)->vec4<f32> {
 let size=vec2<i32>(textureDimensions(map,level));
 let p=clamp(uv,vec2(0.),vec2(1.))*vec2<f32>(size)-0.5;
 let base=vec2<i32>(floor(p));let weight=fract(p);
 let low=vec2<i32>(0);let high=size-vec2<i32>(1);
 let a=textureLoad(map,clamp(base,low,high),level);
 let b=textureLoad(map,clamp(base+vec2<i32>(1,0),low,high),level);
 let c=textureLoad(map,clamp(base+vec2<i32>(0,1),low,high),level);
 let d=textureLoad(map,clamp(base+vec2<i32>(1,1),low,high),level);
 return mix(mix(a,b,weight.x),mix(c,d,weight.x),weight.y);
}
fn texel_trilinear(map:texture_2d<f32>, uv:vec2<f32>, footprint:f32)->vec4<f32> {
 let last=i32(textureNumLevels(map))-1;
 let lod=clamp(log2(max(footprint*f32(textureDimensions(map).x),1e-10)),0.,f32(last));
 let lower=i32(floor(lod));let upper=min(lower+1,last);
 return mix(texel_bilinear(map,uv,lower),texel_bilinear(map,uv,upper),fract(lod));
}
