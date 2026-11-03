// Exact min reduction: four4px tiles span the original16px tile.
@group(0) @binding(0) var fine:texture_2d<f32>;
@group(0) @binding(1) var tiles:texture_storage_2d<r32float,write>;
@compute @workgroup_size(8,8) fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if any(id.xy>=textureDimensions(tiles)) {return;}
    let size=textureDimensions(fine);
    var farthest=1.0;
    for(var y=0u;y<4u;y++) {for(var x=0u;x<4u;x++) {
        let p=id.xy*4u+vec2<u32>(x,y);
        var d=0.0;
        if all(p<size) {d=textureLoad(fine,vec2<i32>(p),0).x;}
        farthest=min(farthest,d);
    }}
    textureStore(tiles,vec2<i32>(id.xy),vec4<f32>(farthest));
}
