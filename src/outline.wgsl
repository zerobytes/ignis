@group(0) @binding(0) var mask:texture_2d<f32>;
@vertex fn vs_edge(@builtin(vertex_index) i:u32) -> @builtin(position) vec4<f32> {
    let p=array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0));
    return vec4<f32>(p[i],0.0,1.0);
}
fn coverage(p:vec2<i32>,size:vec2<i32>)->f32 {
    if any(p<vec2<i32>(0)) || any(p>=size) {return 0.0;}
    return textureLoad(mask,p,0).r;
}
@fragment fn fs_edge(@builtin(position) position:vec4<f32>) -> @location(0) vec4<f32> {
    let p=vec2<i32>(position.xy);let size=vec2<i32>(textureDimensions(mask));
    if coverage(p,size)<0.5 {discard;}
    var inside=1.0;
    for(var y=-2;y<=2;y++){for(var x=-2;x<=2;x++){
        inside=min(inside,coverage(p+vec2<i32>(x,y),size));
    }}
    if inside>0.5 {discard;}
    return vec4<f32>(1.0,0.55,0.2,1.0);
}
