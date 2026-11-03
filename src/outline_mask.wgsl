struct Camera { vp:mat4x4<f32>, };
@group(0) @binding(0) var<uniform> camera:Camera;
@vertex fn vs_mask(@location(0) position:vec3<f32>) -> @builtin(position) vec4<f32> {
    return camera.vp * vec4<f32>(position,1.0);
}
@fragment fn fs_mask() -> @location(0) f32 {return 1.0;}
