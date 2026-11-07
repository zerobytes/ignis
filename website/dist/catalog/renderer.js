// Browser surface host for the unmodified Ignis scene shader and native GPU packets.
// This host does not implement a second material evaluator or approximate the recipe.
const cap = 560_000;
const normalize = a => {const n=Math.hypot(...a); return a.map(v=>v/n);};
const cross=(a,b)=>[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
const dot=(a,b)=>a.reduce((n,v,i)=>n+v*b[i],0);
function multiply(a,b){const m=new Float32Array(16);for(let c=0;c<4;c++)for(let r=0;r<4;r++)for(let k=0;k<4;k++)m[c*4+r]+=a[k*4+r]*b[c*4+k];return m;}
function camera(yaw,pitch,distance,aspect,center=[0,0,0],radius=1){
 distance*=radius;
 const eye=[Math.sin(yaw)*Math.cos(pitch)*distance,Math.sin(pitch)*distance,Math.cos(yaw)*Math.cos(pitch)*distance];
 const z=normalize(eye),x=normalize(cross([0,1,0],z)),y=cross(z,x);
 for(let i=0;i<3;i++)eye[i]+=center[i];
 const view=new Float32Array([x[0],y[0],z[0],0,x[1],y[1],z[1],0,x[2],y[2],z[2],0,-dot(x,eye),-dot(y,eye),-dot(z,eye),1]);
 const f=1/Math.tan(Math.PI/8),n=Math.min(.01*radius,distance*.02),far=100*radius;
 // Floating-point reverse depth preserves bark/layer separation during close inspection.
 const projection=new Float32Array([f/aspect,0,0,0,0,f,0,0,0,0,n/(far-n),-1,0,0,n*far/(far-n),0]);
 return {matrix:multiply(projection,view),eye,x,y,z,n,far,f,aspect};
}
function geometry(shape,plane){
 const vertices=[],indices=[];
 const vertex=(p,n,u,v)=>{const rest=plane==='xz'?[u,0,v]:plane==='yz'?[0,u,v]:[u,v,0];vertices.push(...p,...n,1,1,1,...rest,0,0,0,0);};
 if(shape==='sphere'){
  const rows=40,cols=64;
  for(let j=0;j<=rows;j++)for(let i=0;i<=cols;i++){const v=j/rows,u=i/cols,t=v*Math.PI,a=u*Math.PI*2;const p=[Math.sin(t)*Math.cos(a),Math.cos(t),Math.sin(t)*Math.sin(a)];vertex(p,p,u,v);}
  for(let j=0;j<rows;j++)for(let i=0;i<cols;i++){const a=j*(cols+1)+i,b=a+cols+1;indices.push(a,a+1,b,a+1,b+1,b);}
 }else{
  const faces=shape==='plane'?[[[0,0,1],[1,0,0],[0,1,0],[0,0,0]]]:[
   [[0,0,1],[1,0,0],[0,1,0],[0,0,.8]],[[0,0,-1],[-1,0,0],[0,1,0],[0,0,-.8]],
   [[1,0,0],[0,0,-1],[0,1,0],[.8,0,0]],[[-1,0,0],[0,0,1],[0,1,0],[-.8,0,0]],
   [[0,1,0],[1,0,0],[0,0,-1],[0,.8,0]],[[0,-1,0],[1,0,0],[0,0,1],[0,-.8,0]]];
  for(const [normal,x,y,center] of faces){const start=vertices.length/16;for(const [u,v] of [[0,0],[1,0],[1,1],[0,1]])vertex(center.map((q,k)=>q+(2*u-1)*x[k]*(shape==='plane'?1.05:.8)+(2*v-1)*y[k]*(shape==='plane'?1.05:.8)),normal,u,v);indices.push(start,start+1,start+2,start,start+2,start+3);}
 }
 const data=new Float32Array(vertices),ids=new Uint32Array(data.buffer);
 for(let i=0;i<vertices.length/16;i++)ids[i*16+12]=0; // u32 material index, native Vertex ABI.
 return {vertices:data,indices:new Uint32Array(indices)};
}
async function verified(url,hash,signal,expected=null,max=1024*1024){
 const response=await fetch(url,{signal});if(!response.ok)throw new Error(`HTTP ${response.status}`);
 const length=Number(response.headers.get('content-length'));
 if(length>max)throw new Error('Preview resource exceeds budget');
 const reader=response.body.getReader(),chunks=[];let size=0;
 try{while(true){const {done,value}=await reader.read();if(done)break;size+=value.length;if(size>max||expected!==null&&size>expected)throw new Error('Preview resource exceeds budget');chunks.push(value);}}catch(error){await reader.cancel();throw error;}
 if(expected!==null&&size!==expected)throw new Error('Preview size mismatch');
 const joined=new Uint8Array(size);let offset=0;for(const chunk of chunks){joined.set(chunk,offset);offset+=chunk.length;}const bytes=joined.buffer;
 const digest=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
 if(digest!==hash)throw new Error('Preview integrity check failed');return bytes;
}

export class IgnisPreview{
 static async create(canvas,manifest,onError){
  if(!navigator.gpu)throw new Error('WEBGPU_UNAVAILABLE');
  const adapter=await navigator.gpu.requestAdapter({powerPreference:'low-power'});
  if(!adapter)throw new Error('WEBGPU_UNAVAILABLE');
  const device=await adapter.requestDevice();
  let host;
  try{
   host=new IgnisPreview(canvas,device,onError);
   const bytes=await verified(new URL('data/ignis.wgsl',import.meta.url),manifest.shader_sha256);
   const shader=device.createShaderModule({label:'Ignis native scene shader',code:new TextDecoder().decode(bytes)});
   const compilation=await shader.getCompilationInfo();const errors=compilation.messages.filter(m=>m.type==='error');
   if(errors.length)throw new Error(errors.map(m=>`${m.lineNum}:${m.linePos} ${m.message}`).join('\n'));
   const format=navigator.gpu.getPreferredCanvasFormat();host.format=`${format}-srgb`;
   host.context.configure({device,format,viewFormats:[host.format],alphaMode:'opaque'});
   host.shader=shader;
   host.pipeline=await device.createRenderPipelineAsync({label:'Ignis material preview',layout:'auto',vertex:{module:shader,entryPoint:'vs_main',buffers:[{arrayStride:64,attributes:[{shaderLocation:0,offset:0,format:'float32x3'},{shaderLocation:1,offset:12,format:'float32x3'},{shaderLocation:2,offset:24,format:'float32x3'},{shaderLocation:3,offset:36,format:'float32x3'},{shaderLocation:4,offset:48,format:'uint32'},{shaderLocation:5,offset:52,format:'float32x3'}]}]},fragment:{module:shader,entryPoint:'fs_main',targets:[{format:host.format}]},primitive:{topology:'triangle-list',cullMode:'none'},depthStencil:{format:'depth32float',depthWriteEnabled:true,depthCompare:'greater'}});
   return host;
  }catch(error){if(host)host.destroy();else device.destroy();throw error;}
 }
 constructor(canvas,device,onError){
  this.canvas=canvas;this.device=device;this.context=canvas.getContext('webgpu');if(!this.context)throw new Error('WEBGPU_UNAVAILABLE');
  this.onError=onError;this.yaw=.35;this.pitch=.12;this.distance=3.6;this.light=35;this.shape='sphere';this.plane='xy';this.frame=0;this.disposed=false;this.visible=true;this.resources=[];this.parts=[];this.center=[0,0,0];this.radius=1;this.mode='material';this.target=[0,0,0];this.navigation='rotate';
  this.uniform=device.createBuffer({size:464,usage:GPUBufferUsage.UNIFORM|GPUBufferUsage.COPY_DST});
  this.shadow=device.createTexture({size:[1,1],format:'depth32float',usage:GPUTextureUsage.TEXTURE_BINDING|GPUTextureUsage.RENDER_ATTACHMENT});
  this.cube=device.createTexture({size:[1,1,6],format:'rgba8unorm',usage:GPUTextureUsage.TEXTURE_BINDING});
  this.lut=device.createTexture({size:[1,1],format:'rgba8unorm',usage:GPUTextureUsage.TEXTURE_BINDING});
  this.abort=new AbortController();const options={signal:this.abort.signal};const pointers=new Map();
  canvas.addEventListener('contextmenu',e=>e.preventDefault(),options);
  canvas.addEventListener('pointerdown',e=>{if(e.button>2)return;canvas.focus({preventScroll:true});pointers.set(e.pointerId,{x:e.clientX,y:e.clientY,pan:e.shiftKey||e.button!==0});canvas.setPointerCapture(e.pointerId);},options);
  canvas.addEventListener('pointermove',e=>{
   const previous=pointers.get(e.pointerId);if(!previous)return;
   const next={...previous,x:e.clientX,y:e.clientY};pointers.set(e.pointerId,next);
   if(pointers.size===2){
    const other=[...pointers.entries()].find(([id])=>id!==e.pointerId)[1];
    const before=Math.hypot(previous.x-other.x,previous.y-other.y),after=Math.hypot(next.x-other.x,next.y-other.y);
    this.pan((next.x-previous.x)/2,(next.y-previous.y)/2);if(before>0&&after>0)this.zoom(before/after);
   }else if(pointers.size===1){
    if(previous.pan||e.shiftKey||this.navigation==='pan')this.pan(next.x-previous.x,next.y-previous.y);
    else{this.yaw-=(next.x-previous.x)*.009;this.pitch=Math.max(-1.55,Math.min(1.55,this.pitch+(next.y-previous.y)*.009));this.invalidate();}
   }
  },options);
  for(const event of ['pointerup','pointercancel','lostpointercapture'])canvas.addEventListener(event,e=>pointers.delete(e.pointerId),options);
  canvas.addEventListener('wheel',e=>{e.preventDefault();const unit=e.deltaMode===1?16:e.deltaMode===2?canvas.clientHeight:1;this.zoom(Math.exp(Math.max(-1,Math.min(1,e.deltaY*unit*.002))));},{...options,passive:false});
  canvas.addEventListener('dblclick',e=>{this.focusAt(e.clientX,e.clientY).catch(error=>{if(!this.disposed)this.onError(error);});},options);
  canvas.addEventListener('keydown',e=>{
   let handled=true;const arrows={ArrowLeft:[-35,0],ArrowRight:[35,0],ArrowUp:[0,-35],ArrowDown:[0,35]};
   if(e.shiftKey&&arrows[e.key])this.pan(...arrows[e.key]);
   else if(e.key==='ArrowLeft')this.yaw-=.12;else if(e.key==='ArrowRight')this.yaw+=.12;
   else if(e.key==='ArrowUp')this.pitch=Math.min(1.55,this.pitch+.12);else if(e.key==='ArrowDown')this.pitch=Math.max(-1.55,this.pitch-.12);
   else if(e.key==='+'||e.key==='=')this.zoom(.8);else if(e.key==='-')this.zoom(1.25);
   else if(e.key.toLowerCase()==='r')this.reset();else handled=false;
   if(handled){e.preventDefault();this.invalidate();}
  },options);
  document.addEventListener('visibilitychange',()=>{if(document.hidden){cancelAnimationFrame(this.frame);this.frame=0;}else this.invalidate();},options);
  this.resize=new ResizeObserver(()=>this.invalidate());this.resize.observe(canvas);
  this.observer=new IntersectionObserver(entries=>{this.visible=entries[0].isIntersecting;if(this.visible)this.invalidate();});this.observer.observe(canvas);
  device.lost.then(info=>{if(!this.disposed){this.destroy();onError(new Error(`GPU_DEVICE_LOST: ${info.reason}`));}});
  device.addEventListener('uncapturederror',event=>{event.preventDefault();if(!this.disposed){this.destroy();onError(event.error);}},{signal:this.abort.signal});
 }
 buffer(data,usage){const b=this.device.createBuffer({size:data.byteLength,usage:usage|GPUBufferUsage.COPY_DST});this.device.queue.writeBuffer(b,0,data);return b;}
 groupFor(pipeline,material,fields){return this.device.createBindGroup({layout:pipeline.getBindGroupLayout(0),entries:[{binding:0,resource:{buffer:this.uniform}},{binding:1,resource:this.shadow.createView()},{binding:2,resource:this.device.createSampler({compare:'less-equal'})},{binding:3,resource:{buffer:material}},{binding:4,resource:this.cube.createView({dimension:'cube'})},{binding:5,resource:this.cube.createView({dimension:'cube'})},{binding:6,resource:this.lut.createView()},{binding:7,resource:this.device.createSampler({magFilter:'linear',minFilter:'linear'})},{binding:8,resource:{buffer:fields}}]});}
 async selectMesh(asset,signal){
  const preview=asset.preview;
  const compressed=await verified(new URL(preview.path,import.meta.url),preview.packet_sha256,signal,preview.packet_bytes,96*1024*1024);
  if(!('DecompressionStream' in window))throw new Error('GZIP_UNAVAILABLE');
  if(!Number.isInteger(preview.decoded_bytes)||preview.decoded_bytes<16||preview.decoded_bytes>192*1024*1024)throw new Error('Mesh size outside budget');
  const packet=new Uint8Array(preview.decoded_bytes);let size=0;
  const reader=new Blob([compressed]).stream().pipeThrough(new DecompressionStream('gzip')).getReader();
  try{while(true){if(signal?.aborted)throw new DOMException('Aborted','AbortError');const {done,value}=await reader.read();if(done)break;size+=value.length;if(size>packet.length)throw new Error('Decoded mesh exceeds budget');packet.set(value,size-value.length);}}catch(error){await reader.cancel();throw error;}
  const digest=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',packet)),b=>b.toString(16).padStart(2,'0')).join('');
  if(size!==packet.length||digest!==preview.decoded_sha256)throw new Error('Decoded mesh integrity failure');
  const header=new DataView(packet.buffer),jsonBytes=header.getUint32(8,true),dataBytes=header.getUint32(12,true);
  if(header.getUint32(0,true)!==0x504d4749||header.getUint32(4,true)!==1||jsonBytes%4||jsonBytes>65536||16+jsonBytes+dataBytes!==size)throw new Error('Unsupported mesh ABI');
  const metadata=JSON.parse(new TextDecoder().decode(packet.subarray(16,16+jsonBytes))),data=packet.subarray(16+jsonBytes);
  if(metadata.vertex_stride!==24||metadata.material_stride!==1392||!Array.isArray(metadata.parts)||!metadata.parts.length||metadata.parts.length>128||metadata.material_bytes!==1392*metadata.parts.length||metadata.program_offset!==metadata.material_bytes||metadata.program_bytes<16||metadata.program_bytes%16)throw new Error('Unsupported static mesh layout');
  const slice=(offset,length)=>{if(!Number.isSafeInteger(offset)||!Number.isSafeInteger(length)||offset<0||length<=0||offset%4||offset+length>data.length)throw new Error('Invalid mesh range');return data.subarray(offset,offset+length);};
  const min=metadata.bounds.min,max=metadata.bounds.max;
  if(!Array.isArray(min)||!Array.isArray(max)||min.length!==3||max.length!==3||min.some((v,i)=>!Number.isFinite(v)||!Number.isFinite(max[i])||v>max[i]))throw new Error('Invalid mesh bounds');
  const center=min.map((v,i)=>(v+max[i])/2),radius=Math.hypot(...min.map((v,i)=>(max[i]-v)/2));
  if(!Number.isFinite(radius)||radius<=0)throw new Error('Empty mesh bounds');
  if(!this.meshPipeline)this.meshPipeline=await this.device.createRenderPipelineAsync({label:'Ignis static mesh preview',layout:'auto',vertex:{module:this.shader,entryPoint:'vs_shared_position_coordinates',buffers:[{arrayStride:24,attributes:[{shaderLocation:0,offset:0,format:'float32x3'},{shaderLocation:1,offset:12,format:'float32x3'}]},{arrayStride:80,stepMode:'instance',attributes:[{shaderLocation:6,offset:0,format:'float32x4'},{shaderLocation:7,offset:16,format:'float32x4'},{shaderLocation:8,offset:32,format:'float32x4'},{shaderLocation:9,offset:48,format:'float32x4'},{shaderLocation:10,offset:64,format:'float32x3'},{shaderLocation:11,offset:76,format:'uint32'}]}]},fragment:{module:this.shader,entryPoint:'fs_main',targets:[{format:this.format}]},primitive:{topology:'triangle-list',cullMode:'none'},depthStencil:{format:'depth32float',depthWriteEnabled:true,depthCompare:'greater'}});
  if(signal?.aborted||this.disposed)throw new DOMException('Aborted','AbortError');
  const resources=[],parts=[];let group;
  const buffer=(bytes,usage)=>{const b=this.buffer(bytes,usage);resources.push(b);return b;};
  try{
   const material=buffer(slice(0,metadata.material_bytes),GPUBufferUsage.STORAGE),fields=buffer(slice(metadata.program_offset,metadata.program_bytes),GPUBufferUsage.STORAGE);
   group=this.groupFor(this.meshPipeline,material,fields);
   for(const part of metadata.parts){if(!Number.isInteger(part.vertices)||part.vertices<=0||!Number.isInteger(part.indices)||part.indices<=0||part.indices%3)throw new Error('Invalid mesh counts');parts.push({vertex:buffer(slice(part.vertex_offset,part.vertices*24),GPUBufferUsage.VERTEX),index:buffer(slice(part.index_offset,part.indices*4),GPUBufferUsage.INDEX),instance:buffer(slice(part.instance_offset,80),GPUBufferUsage.VERTEX),count:part.indices});}
  }catch(error){resources.forEach(r=>r.destroy());throw error;}
  this.resources.forEach(r=>r.destroy());this.resources=resources;this.parts=parts;this.group=group;this.mode='mesh';this.asset=asset.id;this.center=center;this.radius=radius;this.reset();
 }
 async select(asset,signal){
  if(asset.kind==='mesh')return this.selectMesh(asset,signal);
  const packet=await verified(new URL(`data/${asset.id}.bin`,import.meta.url),asset.preview.packet_sha256,signal);
  if(signal?.aborted||this.disposed)throw new DOMException('Aborted','AbortError');
  const h=new DataView(packet);const table=h.getUint32(8,true),program=h.getUint32(12,true);
  if(h.getUint32(0,true)!==0x50574749||h.getUint32(4,true)!==1||table!==1392||program<16||program%16||packet.byteLength!==16+table+program)throw new Error('Unsupported Ignis preview ABI');
  const material=this.buffer(new Uint8Array(packet,16,table),GPUBufferUsage.STORAGE);
  const fields=this.buffer(new Uint8Array(packet,16+table,program),GPUBufferUsage.STORAGE);
  const resources=[material,fields];let group;
  try{group=this.device.createBindGroup({layout:this.pipeline.getBindGroupLayout(0),entries:[{binding:0,resource:{buffer:this.uniform}},{binding:1,resource:this.shadow.createView()},{binding:2,resource:this.device.createSampler({compare:'less-equal'})},{binding:3,resource:{buffer:material}},{binding:4,resource:this.cube.createView({dimension:'cube'})},{binding:5,resource:this.cube.createView({dimension:'cube'})},{binding:6,resource:this.lut.createView()},{binding:7,resource:this.device.createSampler({magFilter:'linear',minFilter:'linear'})},{binding:8,resource:{buffer:fields}}]});}catch(e){resources.forEach(r=>r.destroy());throw e;}
  this.resources.forEach(r=>r.destroy());this.resources=resources;this.group=group;this.asset=asset.id;this.plane=asset.preview.plane;this.parts=[];const changed=this.mode!=='material';this.mode='material';this.center=[0,0,0];this.radius=1;if(changed)this.reset();
  this.setShape(this.shape);this.invalidate();
 }
 setShape(shape){this.shape=shape;if(!this.group||this.mode==='mesh')return;const key=`${shape}:${this.plane}`;if(this.geometryKey===key){this.invalidate();return;}const mesh=geometry(shape,this.plane);const vb=this.buffer(mesh.vertices,GPUBufferUsage.VERTEX),ib=this.buffer(mesh.indices,GPUBufferUsage.INDEX);this.vertex?.destroy();this.index?.destroy();this.vertex=vb;this.index=ib;this.count=mesh.indices.length;this.geometryKey=key;this.invalidate();}
 async focusAt(clientX,clientY){
  if(!this.group||this.disposed)return;
  cancelAnimationFrame(this.frame);this.frame=0;this.draw();
  const view=this.lastCamera,draw=this.draws,asset=this.asset,rect=this.canvas.getBoundingClientRect();
  const u=Math.max(0,Math.min(.999999,(clientX-rect.left)/rect.width)),v=Math.max(0,Math.min(.999999,(clientY-rect.top)/rect.height));
  const pixelX=Math.floor(u*this.canvas.width),pixelY=Math.floor(v*this.canvas.height);
  // Read one visible depth on demand; no retained CPU mesh or per-frame readback.
  if(!this.focusPipeline)this.focusPipeline=this.device.createComputePipeline({layout:'auto',compute:{module:this.device.createShaderModule({code:`
   @group(0) @binding(0) var depth: texture_depth_2d;
   @group(0) @binding(1) var<uniform> pixel: vec2u;
   @group(0) @binding(2) var<storage,read_write> result: array<f32>;
   @compute @workgroup_size(1) fn main(){result[0]=textureLoad(depth,vec2i(pixel),0);}
  `}),entryPoint:'main'}});
  const readback=this.device.createBuffer({size:4,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
  const sample=this.device.createBuffer({size:4,usage:GPUBufferUsage.STORAGE|GPUBufferUsage.COPY_SRC}),pixel=this.buffer(new Uint32Array([pixelX,pixelY]),GPUBufferUsage.UNIFORM);
  try{
   const encoder=this.device.createCommandEncoder();
   const group=this.device.createBindGroup({layout:this.focusPipeline.getBindGroupLayout(0),entries:[{binding:0,resource:this.depth.createView()},{binding:1,resource:{buffer:pixel}},{binding:2,resource:{buffer:sample}}]});
   const pass=encoder.beginComputePass();pass.setPipeline(this.focusPipeline);pass.setBindGroup(0,group);pass.dispatchWorkgroups(1);pass.end();encoder.copyBufferToBuffer(sample,0,readback,0,4);this.device.queue.submit([encoder.finish()]);
   await readback.mapAsync(GPUMapMode.READ);const depth=new Float32Array(readback.getMappedRange())[0];
   if(this.disposed||this.asset!==asset||this.draws!==draw||this.frame||!(depth>0))return;
   const z=view.n*view.far/(view.n+depth*(view.far-view.n));
   const x=(2*(pixelX+.5)/this.canvas.width-1)*z*view.aspect/view.f,y=(1-2*(pixelY+.5)/this.canvas.height)*z/view.f;
   this.target=view.eye.map((value,i)=>value-view.z[i]*z+view.x[i]*x+view.y[i]*y);
   const offset=view.eye.map((value,i)=>value-this.target[i]),length=Math.hypot(...offset);
   this.distance=Math.max(.0005,length/this.radius);this.yaw=Math.atan2(offset[0],offset[2]);this.pitch=Math.asin(offset[1]/length);this.invalidate();
  }finally{readback.destroy();sample.destroy();pixel.destroy();}
 }
 zoom(factor){this.distance=Math.max(.0005,Math.min(32,this.distance*factor));this.invalidate();}
 pan(dx,dy){
  const scale=2*this.distance*this.radius*Math.tan(Math.PI/8)/Math.max(1,this.canvas.clientHeight);
  const right=[Math.cos(this.yaw),0,-Math.sin(this.yaw)],up=[-Math.sin(this.yaw)*Math.sin(this.pitch),Math.cos(this.pitch),-Math.cos(this.yaw)*Math.sin(this.pitch)];
  this.target=this.target.map((v,i)=>v+(-dx*right[i]+dy*up[i])*scale);this.invalidate();
 }
 setNavigation(mode){this.navigation=mode;}
 reset(){this.yaw=.35;this.pitch=.12;this.distance=this.mode==='mesh'?2.8:3.6;this.target=[...this.center];this.invalidate();}
 setLight(value){this.light=value;this.invalidate();}
 invalidate(){if(!this.frame&&!this.disposed&&this.visible&&!document.hidden)this.frame=requestAnimationFrame(()=>{this.frame=0;try{this.draw();}catch(error){this.destroy();this.onError(error);}});}
 draw(){
  if(!this.group||this.disposed)return;
  const rect=this.canvas.getBoundingClientRect();const dpr=Math.min(devicePixelRatio||1,1.5);let w=Math.max(1,Math.round(rect.width*dpr)),h=Math.max(1,Math.round(rect.height*dpr));const factor=Math.min(1,Math.sqrt(cap/(w*h)),this.device.limits.maxTextureDimension2D/Math.max(w,h));w=Math.max(1,Math.floor(w*factor));h=Math.max(1,Math.floor(h*factor));
  if(this.canvas.width!==w||this.canvas.height!==h||!this.depth){this.canvas.width=w;this.canvas.height=h;this.depth?.destroy();this.depth=this.device.createTexture({size:[w,h],format:'depth32float',usage:GPUTextureUsage.RENDER_ATTACHMENT|GPUTextureUsage.TEXTURE_BINDING});}
  const view=camera(this.yaw,this.pitch,this.distance,w/h,this.target,this.radius);this.lastCamera=view;const {matrix,eye}=view;const globals=new Float32Array(116);globals.set(matrix);globals.set([1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],16);
  const angle=this.light*Math.PI/180;globals.set([-Math.sin(angle),-.65,-Math.cos(angle),.22],32);globals.set([...eye,0],40);globals.set([0,0,1,1],44);globals.set([1,1,1,1],112);this.device.queue.writeBuffer(this.uniform,0,globals);
  const encoder=this.device.createCommandEncoder();const pass=encoder.beginRenderPass({colorAttachments:[{view:this.context.getCurrentTexture().createView({format:this.format}),clearValue:{r:.022,g:.029,b:.025,a:1},loadOp:'clear',storeOp:'store'}],depthStencilAttachment:{view:this.depth.createView(),depthClearValue:0,depthLoadOp:'clear',depthStoreOp:'store'}});
  pass.setPipeline(this.mode==='mesh'?this.meshPipeline:this.pipeline);pass.setBindGroup(0,this.group);if(this.mode==='mesh'){for(const part of this.parts){pass.setVertexBuffer(0,part.vertex);pass.setVertexBuffer(1,part.instance);pass.setIndexBuffer(part.index,'uint32');pass.drawIndexed(part.count);}}else{pass.setVertexBuffer(0,this.vertex);pass.setIndexBuffer(this.index,'uint32');pass.drawIndexed(this.count);}pass.end();this.device.queue.submit([encoder.finish()]);this.draws=(this.draws||0)+1;this.drawnAsset=this.asset;
 }
 diagnostics(){return {asset:this.asset,drawnAsset:this.drawnAsset,draws:this.draws||0,pendingFrame:!!this.frame,width:this.canvas.width,height:this.canvas.height,shape:this.mode==='mesh'?'mesh':this.shape,parts:this.parts.length,radius:this.radius,disposed:this.disposed};}
 destroy(){if(this.disposed)return;this.disposed=true;cancelAnimationFrame(this.frame);this.frame=0;this.abort.abort();this.resize?.disconnect();this.observer?.disconnect();for(const r of [...this.resources,this.vertex,this.index,this.depth,this.uniform,this.shadow,this.cube,this.lut])r?.destroy();this.context?.unconfigure();this.device.destroy();}
}
