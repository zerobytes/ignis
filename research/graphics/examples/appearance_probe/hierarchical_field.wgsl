@group(1) @binding(2) var height_bounds:texture_2d<f32>;
// Bounds contain all source triangles in each region; no averaged height is used.
fn narrow_interval(origin:vec3<f32>,ray:vec3<f32>,initial:vec2<f32>)->vec3<f32>{
    var interval=initial;var steps=0u;
    let grid=vec2<f32>(textureDimensions(heights)-vec2(1u));
    for(var round=0u;round<min(u32(field.size.z),4u);round+=1u){
        let a=origin.xz+ray.xz*interval.x;let b=origin.xz+ray.xz*interval.y;
        let low=clamp((min(a,b)/12.+.5)*grid-vec2(.002),vec2(0.),grid);
        let high=clamp((max(a,b)/12.+.5)*grid+vec2(.002),vec2(0.),grid);
        let span=max(high.x-low.x,high.y-low.y)/8.;
        let level=min(u32(ceil(log2(max(span,1.)))),textureNumLevels(height_bounds)-1u);
        let width=8.*exp2(f32(level));
        let last=vec2<i32>(ceil(grid/width))-vec2(1);
        let first=min(vec2<i32>(floor(low/width)),last);let end=min(vec2<i32>(floor(high/width)),last);
        let v0=textureLoad(height_bounds,first,i32(level)).xy;
        let v1=textureLoad(height_bounds,vec2(end.x,first.y),i32(level)).xy;
        let v2=textureLoad(height_bounds,vec2(first.x,end.y),i32(level)).xy;
        let v3=textureLoad(height_bounds,end,i32(level)).xy;
        let minimum=min(min(v0.x,v1.x),min(v2.x,v3.x));let maximum=max(max(v0.y,v1.y),max(v2.y,v3.y));
        let near=(maximum-origin.y)/ray.y;let far=(minimum-origin.y)/ray.y;
        // Keep enough slack for world-coordinate ray arithmetic in f32.
        let slack=max(abs(interval.x),abs(interval.y))*.000001;
        let next=vec2(max(interval.x,near-slack),min(interval.y,far+slack));steps+=1u;
        if next.x>next.y {return vec3(next,f32(steps));}
        let ratio=(next.y-next.x)/max(interval.y-interval.x,1e-10);
        interval=next;
        if level==0u || ratio>.9 {break;}
    }
    return vec3(interval,f32(steps));
}
