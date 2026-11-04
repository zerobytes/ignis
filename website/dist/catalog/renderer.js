// Browser surface host for the unmodified Ignis scene shader and native GPU packets.
// This host does not implement a second material evaluator or approximate the recipe.
const cap = 560_000;
const normalize = a => {const n=Math.hypot(...a); return a.map(v=>v/n);};
const cross=(a,b)=>[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
const dot=(a,b)=>a.reduce((n,v,i)=>n+v*b[i],0);
function multiply(a,b){const m=new Float32Array(16);for(let c=0;c<4;c++)for(let r=0;r<4;r++)for(let k=0;k<4;k++)m[c*4+r]+=a[k*4+r]*b[c*4+k];return m;}
function camera(yaw,pitch,distance,aspect){
 const eye=[Math.sin(yaw)*Math.cos(pitch)*distance,Math.sin(pitch)*distance,Math.cos(yaw)*Math.cos(pitch)*distance];
 const z=normalize(eye),x=normalize(cross([0,1,0],z)),y=cross(z,x);
 const view=new Float32Array([x[0],y[0],z[0],0,x[1],y[1],z[1],0,x[2],y[2],z[2],0,-dot(x,eye),-dot(y,eye),-dot(z,eye),1]);
 const f=1/Math.tan(Math.PI/8),n=.03,far=100;
 const projection=new Float32Array([f/aspect,0,0,0,0,f,0,0,0,0,far/(n-far),-1,0,0,n*far/(n-far),0]);
 return {matrix:multiply(projection,view),eye};
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
async function verified(url,hash,signal){
 const response=await fetch(url,{signal});if(!response.ok)throw new Error(`HTTP ${response.status}`);
 const bytes=await response.arrayBuffer();if(bytes.byteLength>1024*1024)throw new Error('Preview resource exceeds budget');
 const digest=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
 if(digest!==hash)throw new Error('Preview integrity check failed');return bytes;
}

export class IgnisPreview{
 static async create(canvas,manifest,onError){
  if(!navigator.gpu)throw new Error('WEBGPU_UNAVAILABLE');
  const adapter=await navigator.gpu.requestAdapter({powerPreference:'low-power'});
  if(!adapter)throw new Error('WEBGPU_UNAVAILABLE');
  const device=await adapter.requestDevice();
  const host=new IgnisPreview(canvas,device,onError);
  try{
   const bytes=await verified(new URL('data/ignis.wgsl',import.meta.url),manifest.shader_sha256);
   const shader=device.createShaderModule({label:'Ignis native scene shader',code:new TextDecoder().decode(bytes)});
   const compilation=await shader.getCompilationInfo();const errors=compilation.messages.filter(m=>m.type==='error');
   if(errors.length)throw new Error(errors.map(m=>`${m.lineNum}:${m.linePos} ${m.message}`).join('\n'));
   const format=navigator.gpu.getPreferredCanvasFormat();host.format=`${format}-srgb`;
   host.context.configure({device,format,viewFormats:[host.format],alphaMode:'opaque'});
   host.pipeline=await device.createRenderPipelineAsync({label:'Ignis material preview',layout:'auto',vertex:{module:shader,entryPoint:'vs_main',buffers:[{arrayStride:64,attributes:[{shaderLocation:0,offset:0,format:'float32x3'},{shaderLocation:1,offset:12,format:'float32x3'},{shaderLocation:2,offset:24,format:'float32x3'},{shaderLocation:3,offset:36,format:'float32x3'},{shaderLocation:4,offset:48,format:'uint32'},{shaderLocation:5,offset:52,format:'float32x3'}]}]},fragment:{module:shader,entryPoint:'fs_main',targets:[{format:host.format}]},primitive:{topology:'triangle-list',cullMode:'none'},depthStencil:{format:'depth24plus',depthWriteEnabled:true,depthCompare:'less'}});
   return host;
  }catch(error){host.destroy();throw error;}
 }
 constructor(canvas,device,onError){
  this.canvas=canvas;this.device=device;this.context=canvas.getContext('webgpu');if(!this.context)throw new Error('WEBGPU_UNAVAILABLE');
  this.onError=onError;this.yaw=.35;this.pitch=.12;this.distance=3.6;this.light=35;this.shape='sphere';this.plane='xy';this.frame=0;this.disposed=false;this.visible=true;this.resources=[];
  this.uniform=device.createBuffer({size:464,usage:GPUBufferUsage.UNIFORM|GPUBufferUsage.COPY_DST});
  this.shadow=device.createTexture({size:[1,1],format:'depth32float',usage:GPUTextureUsage.TEXTURE_BINDING|GPUTextureUsage.RENDER_ATTACHMENT});
  this.cube=device.createTexture({size:[1,1,6],format:'rgba8unorm',usage:GPUTextureUsage.TEXTURE_BINDING});
  this.lut=device.createTexture({size:[1,1],format:'rgba8unorm',usage:GPUTextureUsage.TEXTURE_BINDING});
  this.abort=new AbortController();const options={signal:this.abort.signal};let drag=null;
  canvas.addEventListener('pointerdown',e=>{drag=[e.clientX,e.clientY];canvas.setPointerCapture(e.pointerId);},options);
  canvas.addEventListener('pointermove',e=>{if(!drag)return;this.yaw-=(e.clientX-drag[0])*.009;this.pitch=Math.max(-1.3,Math.min(1.3,this.pitch+(e.clientY-drag[1])*.009));drag=[e.clientX,e.clientY];this.invalidate();},options);
  for(const event of ['pointerup','pointercancel','lostpointercapture'])canvas.addEventListener(event,()=>{drag=null;},options);
  canvas.addEventListener('wheel',e=>{e.preventDefault();this.distance=Math.max(2.3,Math.min(6,this.distance+e.deltaY*.003));this.invalidate();},{...options,passive:false});
  canvas.addEventListener('keydown',e=>{let handled=true;if(e.key==='ArrowLeft')this.yaw-=.12;else if(e.key==='ArrowRight')this.yaw+=.12;else if(e.key==='ArrowUp')this.pitch=Math.min(1.3,this.pitch+.12);else if(e.key==='ArrowDown')this.pitch=Math.max(-1.3,this.pitch-.12);else if(e.key==='+'||e.key==='=')this.distance=Math.max(2.3,this.distance-.2);else if(e.key==='-')this.distance=Math.min(6,this.distance+.2);else if(e.key.toLowerCase()==='r')this.reset();else handled=false;if(handled){e.preventDefault();this.invalidate();}},options);
  document.addEventListener('visibilitychange',()=>{if(document.hidden){cancelAnimationFrame(this.frame);this.frame=0;}else this.invalidate();},options);
  this.resize=new ResizeObserver(()=>this.invalidate());this.resize.observe(canvas);
  this.observer=new IntersectionObserver(entries=>{this.visible=entries[0].isIntersecting;if(this.visible)this.invalidate();});this.observer.observe(canvas);
  device.lost.then(info=>{if(!this.disposed){this.destroy();onError(new Error(`GPU_DEVICE_LOST: ${info.reason}`));}});
  device.addEventListener('uncapturederror',event=>{event.preventDefault();if(!this.disposed){this.destroy();onError(event.error);}},{signal:this.abort.signal});
 }
 buffer(data,usage){const b=this.device.createBuffer({size:data.byteLength,usage:usage|GPUBufferUsage.COPY_DST});this.device.queue.writeBuffer(b,0,data);return b;}
 async select(asset,signal){
  const packet=await verified(new URL(`data/${asset.id}.bin`,import.meta.url),asset.preview.packet_sha256,signal);
  if(signal?.aborted||this.disposed)throw new DOMException('Aborted','AbortError');
  const h=new DataView(packet);const table=h.getUint32(8,true),program=h.getUint32(12,true);
  if(h.getUint32(0,true)!==0x50574749||h.getUint32(4,true)!==1||table!==1392||program<16||program%16||packet.byteLength!==16+table+program)throw new Error('Unsupported Ignis preview ABI');
  const material=this.buffer(new Uint8Array(packet,16,table),GPUBufferUsage.STORAGE);
  const fields=this.buffer(new Uint8Array(packet,16+table,program),GPUBufferUsage.STORAGE);
  const resources=[material,fields];let group;
  try{group=this.device.createBindGroup({layout:this.pipeline.getBindGroupLayout(0),entries:[{binding:0,resource:{buffer:this.uniform}},{binding:1,resource:this.shadow.createView()},{binding:2,resource:this.device.createSampler({compare:'less-equal'})},{binding:3,resource:{buffer:material}},{binding:4,resource:this.cube.createView({dimension:'cube'})},{binding:5,resource:this.cube.createView({dimension:'cube'})},{binding:6,resource:this.lut.createView()},{binding:7,resource:this.device.createSampler({magFilter:'linear',minFilter:'linear'})},{binding:8,resource:{buffer:fields}}]});}catch(e){resources.forEach(r=>r.destroy());throw e;}
  this.resources.forEach(r=>r.destroy());this.resources=resources;this.group=group;this.asset=asset.id;this.plane=asset.preview.plane;
  this.setShape(this.shape);this.invalidate();
 }
 setShape(shape){this.shape=shape;if(!this.group)return;const mesh=geometry(shape,this.plane);const vb=this.buffer(mesh.vertices,GPUBufferUsage.VERTEX),ib=this.buffer(mesh.indices,GPUBufferUsage.INDEX);this.vertex?.destroy();this.index?.destroy();this.vertex=vb;this.index=ib;this.count=mesh.indices.length;this.invalidate();}
 reset(){this.yaw=.35;this.pitch=.12;this.distance=3.6;this.invalidate();}
 setLight(value){this.light=value;this.invalidate();}
 invalidate(){if(!this.frame&&!this.disposed&&this.visible&&!document.hidden)this.frame=requestAnimationFrame(()=>{this.frame=0;try{this.draw();}catch(error){this.destroy();this.onError(error);}});}
 draw(){
  if(!this.group||this.disposed)return;
  const rect=this.canvas.getBoundingClientRect();const dpr=Math.min(devicePixelRatio||1,1.5);let w=Math.max(1,Math.round(rect.width*dpr)),h=Math.max(1,Math.round(rect.height*dpr));const factor=Math.min(1,Math.sqrt(cap/(w*h)),this.device.limits.maxTextureDimension2D/Math.max(w,h));w=Math.max(1,Math.floor(w*factor));h=Math.max(1,Math.floor(h*factor));
  if(this.canvas.width!==w||this.canvas.height!==h||!this.depth){this.canvas.width=w;this.canvas.height=h;this.depth?.destroy();this.depth=this.device.createTexture({size:[w,h],format:'depth24plus',usage:GPUTextureUsage.RENDER_ATTACHMENT});}
  const {matrix,eye}=camera(this.yaw,this.pitch,this.distance,w/h);const globals=new Float32Array(116);globals.set(matrix);globals.set([1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],16);
  const angle=this.light*Math.PI/180;globals.set([-Math.sin(angle),-.65,-Math.cos(angle),.22],32);globals.set([...eye,0],40);globals.set([0,0,1,1],44);globals.set([1,1,1,1],112);this.device.queue.writeBuffer(this.uniform,0,globals);
  const encoder=this.device.createCommandEncoder();const pass=encoder.beginRenderPass({colorAttachments:[{view:this.context.getCurrentTexture().createView({format:this.format}),clearValue:{r:.022,g:.029,b:.025,a:1},loadOp:'clear',storeOp:'store'}],depthStencilAttachment:{view:this.depth.createView(),depthClearValue:1,depthLoadOp:'clear',depthStoreOp:'store'}});
  pass.setPipeline(this.pipeline);pass.setBindGroup(0,this.group);pass.setVertexBuffer(0,this.vertex);pass.setIndexBuffer(this.index,'uint32');pass.drawIndexed(this.count);pass.end();this.device.queue.submit([encoder.finish()]);this.draws=(this.draws||0)+1;
 }
 diagnostics(){return {asset:this.asset,draws:this.draws||0,pendingFrame:!!this.frame,width:this.canvas.width,height:this.canvas.height,shape:this.shape,disposed:this.disposed};}
 destroy(){if(this.disposed)return;this.disposed=true;cancelAnimationFrame(this.frame);this.frame=0;this.abort.abort();this.resize?.disconnect();this.observer?.disconnect();for(const r of [...this.resources,this.vertex,this.index,this.depth,this.uniform,this.shadow,this.cube,this.lut])r?.destroy();this.context?.unconfigure();this.device.destroy();}
}
