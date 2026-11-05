struct FieldParams { inverse:mat4x4<f32>,bounds:vec4<f32>,size:vec4<f32> }
@group(1) @binding(0) var heights:texture_2d<f32>;
@group(1) @binding(1) var<uniform> field:FieldParams;
@vertex fn field_vertex(@builtin(vertex_index) i:u32)->@builtin(position) vec4<f32>{
    let p=vec2<f32>(f32((i<<1u)&2u),f32(i&2u));return vec4(p*2.-1.,0.,1.);
}
// Decode the actual source cell: retain its diagonal, never bilinearly smooth height.
fn plane_at(xz:vec2<f32>)->vec4<f32>{
    let n=vec2<i32>(textureDimensions(heights))-vec2(1);
    let uv=clamp(xz/12.+.5,vec2(0.),vec2(1.))*vec2<f32>(n);
    var k=min(vec2<i32>(floor(uv)),n-vec2(1));
    // Source positions are f32. Correct rare arithmetic boundary disagreement.
    var p=(vec2<f32>(k)/vec2<f32>(n)-.5)*12.;
    if xz.x<p.x && k.x>0 {k.x-=1;}
    if xz.y<p.y && k.y>0 {k.y-=1;}
    p=(vec2<f32>(k)/vec2<f32>(n)-.5)*12.;
    var q=(vec2<f32>(k+vec2(1))/vec2<f32>(n)-.5)*12.;
    if xz.x>q.x && k.x<n.x-1 {k.x+=1;}
    if xz.y>q.y && k.y<n.y-1 {k.y+=1;}
    p=(vec2<f32>(k)/vec2<f32>(n)-.5)*12.;q=(vec2<f32>(k+vec2(1))/vec2<f32>(n)-.5)*12.;
    let frac=(xz-p)/(q-p);
    let h10=textureLoad(heights,k+vec2(1,0),0).x;
    let h01=textureLoad(heights,k+vec2(0,1),0).x;
    var h=textureLoad(heights,k,0).x;
    var slope=(vec2(h10,h01)-h)/(q-p);
    if frac.x+frac.y>1. {
        h=textureLoad(heights,k+vec2(1),0).x;
        slope=(h-vec2(h01,h10))/(q-p);p=q;
    }
    return vec4(slope,h-dot(slope,p),h+dot(slope,xz-p));
}
struct FieldPixel { @location(0) color:vec4<f32>,@builtin(frag_depth) depth:f32 }
@fragment fn field_fragment(@builtin(position) screen:vec4<f32>)->FieldPixel {
    let clip=vec4(screen.xy/field.size.xy*vec2(2.,-2.)+vec2(-1.,1.),1.,1.);
    let direction=field.inverse*clip;
    let origin=g.eye.xyz;let ray=normalize(direction.xyz);
    // Exact ray/AABB intersection; epsilon only expands vertical bounds.
    let inv=1./ray;
    let a=(vec3(-6.,field.bounds.x-.000001,-6.)-origin)*inv;
    let b=(vec3(6.,field.bounds.y+.000001,6.)-origin)*inv;
    let lo=min(a,b);let hi=max(a,b);
    var begin=max(max(lo.x,lo.y),max(lo.z,0.));var end=min(min(hi.x,hi.y),hi.z);
    if begin>end {discard;}
    var left=origin+ray*begin;var right=origin+ray*end;
    // Monotone certificate on CPU guarantees at most one crossing, no hidden nearer root.
    if left.y < plane_at(left.xz).w || right.y > plane_at(right.xz).w {discard;}
    var t=(begin+end)*.5;
    var world=origin+ray*t;
    for(var iteration=0u;iteration<32u;iteration+=1u){
        world=origin+ray*t;
        let p=plane_at(world.xz);
        let exact=(p.z+dot(p.xy,origin.xz)-origin.y)/(ray.y-dot(p.xy,ray.xz));
        let hit=origin+ray*exact;
        let hp=plane_at(hit.xz);
        if exact>=begin && exact<=end && all(hp.xyz == p.xyz) {
            t=exact;break;
        }
        if world.y>p.w {begin=t;}else{end=t;}
        t=(begin+end)*.5;
    }
    world=origin+ray*t;
    let projected=g.matrix*vec4(world,1.);
    var out:FieldPixel;
    out.color=material(source_normal(world),world);
    out.depth=projected.z/projected.w;
    return out;
}
