@group(0) @binding(0) var source: texture_2d<f32>;
@vertex fn vs(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
 let uv=vec2<f32>(f32((id<<1u)&2u),f32(id&2u));return vec4<f32>(uv*2.0-1.0,0.0,1.0);
}
@fragment fn fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
 let start=vec2<i32>(p.xy)*4;var value=vec4<f32>(0.0);
 for(var y=0;y<4;y++){for(var x=0;x<4;x++){value+=textureLoad(source,start+vec2<i32>(x,y),0);}}
 return value/16.0;
}
