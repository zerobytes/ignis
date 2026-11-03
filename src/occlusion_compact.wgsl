override fine_tiles:bool=false;
struct Instance { world:mat4x4<f32>, color:vec3<f32>, material:u32 };
struct Bound { rect:vec4<f32>, nearest:f32, a:f32, b:f32, c:f32 };
struct Job { start:u32, count:u32, vertices:u32, indexed:u32 };
struct Args { count:u32, instances:u32, first:u32, base:u32, instance:u32, a:u32, b:u32, c:u32 };
@group(0) @binding(0) var tiles:texture_2d<f32>;
@group(0) @binding(6) var fine:texture_2d<f32>;
@group(0) @binding(1) var<storage,read> source:array<Instance>;
@group(0) @binding(2) var<storage,read> bounds:array<Bound>;
@group(0) @binding(3) var<storage,read> jobs:array<Job>;
@group(0) @binding(4) var<storage,read_write> output:array<Instance>;
@group(0) @binding(5) var<storage,read_write> args:array<Args>;
fn visible(b:Bound)->bool {
    // NaN/infinity/near-plane/large bounds fail open. Stored rectangle is
    // expanded in CPU using transform roundoff and two physical pixels.
    if !(b.nearest>0.0 && b.nearest<1.0) { return true; }
    let size=vec2<f32>(textureDimensions(tiles));
    var lo=clamp(floor(b.rect.xy/16.0),vec2<f32>(0),size-1.0);
    var hi=clamp(floor(b.rect.zw/16.0),vec2<f32>(0),size-1.0);
    var use_fine=false;
    if fine_tiles {
        let fine_size=vec2<f32>(textureDimensions(fine));
        let fine_lo=clamp(floor(b.rect.xy/4.0),vec2<f32>(0),fine_size-1.0);
        let fine_hi=clamp(floor(b.rect.zw/4.0),vec2<f32>(0),fine_size-1.0);
        if all(fine_lo<=fine_hi) && all(fine_hi-fine_lo<=vec2<f32>(8.0)) {
            lo=fine_lo; hi=fine_hi; use_fine=true;
        }
    }
    if !all(lo<=hi) || any(hi-lo>vec2<f32>(8.0)) { return true; }
    var farthest=1.0;
    for(var y=i32(lo.y); y<=i32(hi.y); y++) { for(var x=i32(lo.x); x<=i32(hi.x); x++) {
        var d=0.0;
        if use_fine {d=textureLoad(fine,vec2<i32>(x,y),0).x;}
        else {d=textureLoad(tiles,vec2<i32>(x,y),0).x;}
        farthest=min(farthest,d);
    }}
    // Relative depth guard works with reverse-Z, unlike a fixed normal-Z epsilon.
    if b.nearest*1.00001 >= farthest { return true; }
    // The main pass still uses conventional float depth. Preserve its distant
    // coplanar/quantization ties, even if reverse-Z could distinguish surfaces.
    let near=0.05;
    let far=30000.0;
    let a=far/(far-near);
    let scale=(near*a)/0.05001;
    let candidate=a-scale*b.nearest;
    let occluder=a-scale*farthest;
    return candidate <= occluder+0.00000048;
}
var<workgroup> flags:array<u32,32>;
var<workgroup> count:u32;
@compute @workgroup_size(32) fn main(@builtin(workgroup_id) g:vec3<u32>, @builtin(local_invocation_index) lane:u32) {
    let job=jobs[g.x];
    if lane==0u { count=0u; }
    workgroupBarrier();
    for(var block=0u; block<job.count; block+=32u) {
        let n=block+lane;
        var yes=false;
        if n<job.count { yes=visible(bounds[job.start+n]); }
        flags[lane]=select(0u,1u,yes);
        workgroupBarrier();
        var prefix=0u;
        for(var i=0u; i<lane; i++) { prefix+=flags[i]; }
        if yes { output[job.start+count+prefix]=source[job.start+n]; }
        workgroupBarrier();
        if lane==31u { count+=prefix+flags[lane]; }
        workgroupBarrier();
    }
    if lane==0u { args[g.x]=Args(job.vertices,count,0u,0u,0u,job.count,job.indexed,0u); }
}
