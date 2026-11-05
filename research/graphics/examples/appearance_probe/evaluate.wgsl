// Immutable mesh distributions; refresh all nodes when light/view/material uniforms change.
@compute @workgroup_size(64) fn evaluate(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let id = invocation.x;
    if id >= arrayLength(&evaluated) { return; }
    evaluated[id] = vec4<f32>(summary(id), 0.0);
}
