// Compiled response indexed by the two invariants of the existing material.
@group(GROUP) @binding(0) var responses:texture_2d<f32>;
@group(GROUP) @binding(1) var response_filter:sampler;
fn symbol_material(n0:vec3<f32>,world:vec3<f32>)->vec4<f32>{
 let n=normalize(n0);let l=normalize(g.light.xyz);let v=normalize(g.eye.xyz-world);let h=normalize(l+v);
 let uv=(vec2(max(dot(n,l),0.0),max(dot(n,h),0.0))*127.0+0.5)/128.0;
 return textureSampleLevel(responses,response_filter,uv,0.0);
}
