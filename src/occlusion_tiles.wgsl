override tile_size:u32=16u;
@group(0) @binding(0) var depth: texture_depth_2d;
@group(0) @binding(1) var tiles: texture_storage_2d<r32float, write>;
@compute @workgroup_size(8,8) fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if any(id.xy >= textureDimensions(tiles)) { return; }
    let size = textureDimensions(depth);
    var farthest = 1.0;
    for (var y=0u; y<tile_size; y++) { for (var x=0u; x<tile_size; x++) {
        let p = id.xy*tile_size + vec2<u32>(x,y);
        var d = 0.0;
        if all(p<size) { d = textureLoad(depth,vec2<i32>(p),0); }
        farthest = min(farthest,d);
    }}
    textureStore(tiles,vec2<i32>(id.xy),vec4<f32>(farthest));
}
