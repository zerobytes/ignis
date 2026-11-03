struct ParticleOut {
    @builtin(position) clip:vec4<f32>,
    @location(0) world:vec3<f32>,
    @location(1) normal:vec3<f32>,
    @location(2) uv:vec2<f32>,
    @location(3) @interpolate(flat) material:u32,
    @location(4) @interpolate(flat) tint:vec4<f32>,
    @location(5) @interpolate(flat) softness:f32,
};
@vertex fn vs_particle(@builtin(vertex_index) index:u32,
    @location(0) center_size:vec4<f32>,@location(1) velocity_angle:vec4<f32>,
    @location(2) color:vec4<f32>,@location(3) shape:vec4<f32>,@location(4) material:vec4<u32>)->ParticleOut {
    let corners=array<vec2<f32>,6>(vec2<f32>(-1.,-1.),vec2<f32>(1.,-1.),vec2<f32>(1.,1.),vec2<f32>(-1.,-1.),vec2<f32>(1.,1.),vec2<f32>(-1.,1.));
    let uv=corners[index];
    let right=normalize(globals.atmosphere_right.xyz);
    let up=normalize(globals.atmosphere_up.xyz);
    var angle=velocity_angle.w;
    let projected=vec2<f32>(dot(velocity_angle.xyz,right),dot(velocity_angle.xyz,up));
    if shape.z>0.5 && dot(projected,projected)>0.000001 {angle+=atan2(projected.y,projected.x)-1.57079632679;}
    let cs=cos(angle);let sn=sin(angle);
    let local=vec2<f32>(uv.x*shape.x,uv.y)*center_size.w*0.5;
    let offset=vec2<f32>(local.x*cs-local.y*sn,local.x*sn+local.y*cs);
    var out:ParticleOut;
    out.world=center_size.xyz+right*offset.x+up*offset.y;
    out.clip=globals.view_projection*vec4<f32>(out.world,1.);
    out.normal=-normalize(globals.atmosphere_forward.xyz);
    out.uv=uv;out.material=material.x;out.tint=color;out.softness=shape.y;
    return out;
}
@fragment fn fs_particle(in:ParticleOut)->TransparentOut {
    let radius=length(in.uv);
    let edge=max(fwidth(radius),0.0001);
    let width=max(in.softness,edge);
    let profile=1.-smoothstep(1.-width,1.,radius);
    var surface:VertexOut;
    surface.clip=in.clip;surface.world=in.world;surface.normal=in.normal;
    surface.color=vec3<f32>(1.);surface.rest=vec3<f32>((in.uv+vec2<f32>(1.))*0.5,0.);
    surface.material=in.material;surface.material_projection_weights=vec3<f32>(1.,0.,0.);
    return transparent_surface(surface,true,profile*in.tint.a,in.tint.rgb);
}
