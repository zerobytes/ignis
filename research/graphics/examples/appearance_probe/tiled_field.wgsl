struct TileParams { inverse:mat4x4<f32>,eye:vec4<f32>,bounds:vec4<f32>,size:vec4<u32>,dispatch:vec4<u32> }
@group(0) @binding(0) var<uniform> tile:TileParams;
@group(0) @binding(1) var bounds_texture:texture_2d<f32>;
@group(0) @binding(2) var<storage,read_write> intervals:array<vec2<f32>>;
// A constant-y cross section of a perspective pixel block is a quadrilateral.
// With ray.y negative at every corner, extrema of x/y and z/y occur at corners.
@compute @workgroup_size(8,8)
fn tiles(@builtin(global_invocation_id) id:vec3<u32>){
    if any(id.xy>=tile.dispatch.zw) {return;}
    let block=id.xy+tile.dispatch.xy;
    let first=vec2<f32>(block*tile.size.z);
    let last=min(first+vec2(f32(tile.size.z)),vec2<f32>(tile.size.xy));
    var slopes:array<vec2<f32>,4>;
    var safe=true;
    for(var i=0u;i<4u;i+=1u){
        let pixel=vec2(select(first.x,last.x,(i&1u)!=0u),select(first.y,last.y,(i&2u)!=0u));
        let d=(tile.inverse*vec4(pixel/vec2<f32>(tile.size.xy)*vec2(2.,-2.)+vec2(-1.,1.),1.,1.)).xyz;
        safe=safe && d.y<-.00001;
        slopes[i]=d.xz/d.y;
    }
    var height=tile.bounds.xy+vec2(-.000002,.000002);
    if safe {
        for(var round=0u;round<4u;round+=1u){
            var low=vec2(1e30);var high=vec2(-1e30);
            for(var i=0u;i<4u;i+=1u){
                let a=tile.eye.xz+slopes[i]*(height.x-tile.eye.y);
                let b=tile.eye.xz+slopes[i]*(height.y-tile.eye.y);
                low=min(low,min(a,b));high=max(high,max(a,b));
            }
            // Expansion includes matrix arithmetic and source-cell boundary rounding.
            low=clamp((low/12.+.5)*tile.bounds.zw-vec2(.01),vec2(0.),tile.bounds.zw);
            high=clamp((high/12.+.5)*tile.bounds.zw+vec2(.01),vec2(0.),tile.bounds.zw);
            let span=max(high.x-low.x,high.y-low.y)/8.;
            let level=min(u32(ceil(log2(max(span,1.)))),textureNumLevels(bounds_texture)-1u);
            let width=8.*exp2(f32(level));
            let last_node=vec2<i32>(ceil(tile.bounds.zw/width))-vec2(1);
            let a=min(vec2<i32>(floor(low/width)),last_node);let b=min(vec2<i32>(floor(high/width)),last_node);
            let v0=textureLoad(bounds_texture,a,i32(level)).xy;
            let v1=textureLoad(bounds_texture,vec2(b.x,a.y),i32(level)).xy;
            let v2=textureLoad(bounds_texture,vec2(a.x,b.y),i32(level)).xy;
            let v3=textureLoad(bounds_texture,b,i32(level)).xy;
            height=vec2(max(height.x,min(min(v0.x,v1.x),min(v2.x,v3.x))-.000002),min(height.y,max(max(v0.y,v1.y),max(v2.y,v3.y))+.000002));
            if level==0u {break;}
        }
    }
    intervals[block.y*tile.size.w+block.x]=height;
}
