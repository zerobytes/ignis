@group(0) @binding(0) var depth:texture_depth_2d;
@group(0) @binding(1) var previous:texture_2d<f32>;
@group(0) @binding(2) var destination:texture_storage_2d<r32float,write>;
@compute @workgroup_size(8,8) fn base(@builtin(global_invocation_id) id:vec3<u32>){
 if any(id.xy>=textureDimensions(destination)){return;}
 let size=textureDimensions(depth);var result=1.0;
 for(var y=0u;y<4u;y++){for(var x=0u;x<4u;x++){let p=id.xy*4u+vec2<u32>(x,y);var d=0.0;if all(p<size){d=textureLoad(depth,vec2<i32>(p),0);}result=min(result,d);}}
 textureStore(destination,vec2<i32>(id.xy),vec4<f32>(result));
}
@compute @workgroup_size(8,8) fn reduce(@builtin(global_invocation_id) id:vec3<u32>){
 if any(id.xy>=textureDimensions(destination)){return;}
 let size=textureDimensions(previous);var result=1.0;
 for(var y=0u;y<2u;y++){for(var x=0u;x<2u;x++){let p=min(id.xy*2u+vec2<u32>(x,y),size-1u);result=min(result,textureLoad(previous,vec2<i32>(p),0).x);}}
 textureStore(destination,vec2<i32>(id.xy),vec4<f32>(result));
}
