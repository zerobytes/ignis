// Reference-driven authoring client. Publishes only through the public MCP command API.
// Dimensions are an explicit working hypothesis, not measurements of the photograph.
export const defaults = {
  porchX: -2.05, backZ: -4.65, frontZ: 2.45, floorY: 0.62,
  eaveY: 3.9, ridgeX: 2.7, ridgeY: 6.6, roofLeft: -2.5, roofRight: 6.1,
  bayX: 4.05, bayZ: 2.05, bayRadius: 1.55, bayEaveY: 3.68, bayApexY: 6.05,
  bayApexOffsetX: -.8,
  chimneyX: 1.35, chimneyZ: -2.75, chimneyHeight: 1.6,
  horizontalScale: 1.25,
  camera: { position: [-11.9, 1.5, 16.5], target: [1.13, 3.03, 0.1], fov_degrees: 27.5 },
};

const add=(a,b)=>a.map((x,i)=>x+b[i]);
const sub=(a,b)=>a.map((x,i)=>x-b[i]);
const mul=(a,s)=>a.map(x=>x*s);
const dot=(a,b)=>a.reduce((s,x,i)=>s+x*b[i],0);
const cross=(a,b)=>[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
const unit=a=>mul(a,1/(Math.hypot(...a)||1));
const lerp=(a,b,t)=>a.map((x,i)=>x+(b[i]-x)*t);
const bez=(a,b,c,d,t)=>a.map((x,i)=>x*(1-t)**3+3*b[i]*(1-t)**2*t+3*c[i]*(1-t)*t*t+d[i]*t**3);
const curve=(a,b,c,d,n=10)=>Array.from({length:n+1},(_,i)=>bez(a,b,c,d,i/n));

class Mesh {
  constructor(id,color,shading='flat',material={}) {
    this.id=id; this.color=color; this.shading=shading; this.material=material;
    this.positions=[]; this.triangles=[];
  }
  v(p) { if(!p.every(Number.isFinite))throw Error('Nonfinite vertex '+this.id); this.positions.push(p); return this.positions.length-1; }
  tri(a,b,c) { if(Math.hypot(...cross(sub(this.positions[b],this.positions[a]),sub(this.positions[c],this.positions[a])))>1e-9)this.triangles.push([a,b,c]); }
  quad(a,b,c,d) { const k=[a,b,c,d].map(p=>this.v(p));this.tri(k[0],k[1],k[2]);this.tri(k[0],k[2],k[3]); }
  box(center,size) {
    const [x,y,z]=center, [w,h,d]=size.map(v=>v/2), k=this.positions.length;
    for(const p of [[-w,-h,-d],[w,-h,-d],[w,h,-d],[-w,h,-d],[-w,-h,d],[w,-h,d],[w,h,d],[-w,h,d]])this.v(add([x,y,z],p));
    for(const t of [[0,2,1],[0,3,2],[4,5,6],[4,6,7],[0,4,7],[0,7,3],[1,2,6],[1,6,5],[0,1,5],[0,5,4],[3,7,6],[3,6,2]])this.tri(...t.map(i=>i+k));
  }
  tube(points,radius,sides=6,normal=null,endRadius=radius) {
    if(points.length<2)return;
    const first=this.positions.length;
    let prior=null;
    for(let i=0;i<points.length;i++) {
      const t=unit(sub(points[Math.min(i+1,points.length-1)],points[Math.max(0,i-1)]));
      let u=normal?unit(cross(t,normal)):unit(cross(t,Math.abs(t[1])<0.92?[0,1,0]:[1,0,0]));
      if(prior&&dot(u,prior)<0)u=mul(u,-1); prior=u;
      const v=unit(cross(t,u)), r=radius+(endRadius-radius)*i/(points.length-1);
      for(let j=0;j<sides;j++)this.v(add(points[i],add(mul(u,r*Math.cos(2*Math.PI*j/sides)),mul(v,r*Math.sin(2*Math.PI*j/sides)))));
    }
    for(let i=0;i<points.length-1;i++)for(let j=0;j<sides;j++) {
      const a=first+i*sides+j,b=first+i*sides+(j+1)%sides,c=b+sides,d=a+sides;
      this.tri(a,c,b);this.tri(a,d,c);
    }
    const a=this.v(points[0]),b=this.v(points.at(-1));
    for(let j=0;j<sides;j++){this.tri(a,first+j,first+(j+1)%sides);this.tri(b,first+(points.length-1)*sides+(j+1)%sides,first+(points.length-1)*sides+j);}
  }
  lathe(cx,cz,profile,sides=10) {
    const first=this.positions.length;
    for(const [y,r] of profile)for(let j=0;j<sides;j++)this.v([cx+r*Math.cos(j*2*Math.PI/sides),y,cz+r*Math.sin(j*2*Math.PI/sides)]);
    for(let i=0;i<profile.length-1;i++)for(let j=0;j<sides;j++){
      const a=first+i*sides+j,b=first+i*sides+(j+1)%sides;
      this.tri(a,b,b+sides);this.tri(a,b+sides,a+sides);
    }
  }
  panel(outline,map,thickness=0.06) {
    const n=outline.length,first=this.positions.length;
    for(const depth of [-thickness/2,thickness/2])for(const p of outline)this.v(map(p[0],p[1],depth));
    for(let i=1;i<n-1;i++){this.tri(first,first+i+1,first+i);this.tri(first+n,first+n+i,first+n+i+1);}
    for(let i=0;i<n;i++){const j=(i+1)%n;this.tri(first+i,first+j,first+n+j);this.tri(first+i,first+n+j,first+n+i);}
  }
  asset() { return {id:this.id,geometry:{positions:this.positions,triangles:this.triangles},shading:this.shading,material:this.material}; }
}

function ogive(w,h,spring,n=10) {
  const a=curve([-w/2,spring],[-w/2,spring+(h-spring)*0.53],[-w*.22,h-(h-spring)*.3],[0,h],n);
  return a.concat(a.slice(0,-1).reverse().map(([x,y])=>[-x,y]));
}
function border(mesh,w,h,spring,map,r=.07,n=10) {
  const top=ogive(w,h,spring,n);
  const line=[[-w/2,0],...top,[w/2,0]];
  mesh.tube(line.map(([x,y])=>map(x,y,0)),r,4,unit(sub(map(0,0,1),map(0,0,0))));
  return top;
}
function tree(mesh,w,h,map,thin=false) {
  const normal=unit(sub(map(0,0,1),map(0,0,0))), r=thin?.017:.032;
  const path=p=>p.map(([x,y])=>map(x*w,y*h,0));
  mesh.tube(path([[0,0],[0,.26],[0,.52],[0,.76],[0,.98]]),r,5,normal,r*.5);
  for(const sign of [-1,1])for(const [base,end,reach] of [[.24,.67,.44],[.52,.9,.3]]) {
    const branch=curve([0,base],[sign*.06,base+.18],[sign*reach,end-.2],[sign*reach,end],7);
    mesh.tube(path(branch),r*.85,5,normal,r*.38);
    for(let j=0;j<(base<.3?3:2);j++) {
      const t=.56+j*.15, q=bez([0,base],[sign*.06,base+.18],[sign*reach,end-.2],[sign*reach,end],t);
      const twig=curve(q,[q[0]+sign*.03,q[1]+.06],[sign*(reach+.05-j*.035),end-.06],[sign*(reach+.01-j*.045),end+.055],4);
      mesh.tube(path(twig),r*.5,4,normal,r*.14);
    }
  }
}

export function buildHouse(options={}) {
  const p={...defaults,...options}, detail=options.detail??false, plants=options.vegetation??false;
  const stoneMat={pattern:{kind:'noise',frequency:[3,5,3],low:[.35,.4,.35],high:[1,1,.96],bump:.018,warp:.25,octaves:3,seed:31,contrast:.65},reflectance:{roughness:.82,f0:.035}};
  const joints=[];
  for(let j=0;j<14;j++)joints.push({start:[-8,j*.29],end:[10,j*.29],widths:[.011,.011]});
  for(let j=0;j<3;j++)for(let k=0;k<6;k++)joints.push({start:[(k-2)*1.15+(j%2)*.58,j*.87],end:[(k-2)*1.15+(j%2)*.58,j*.87+.29],widths:[.01,.01]});
  const masonryMat={...stoneMat,detail:{plane:'xy',tint:[.3,.28,.22],opacity:.6,bump:-.012,segments:joints}};
  const slateMat={pattern:{kind:'noise',frequency:[8,5,9],low:[.5,.6,.62],high:[1,.98,.86],bump:.005,warp:.2,octaves:2,seed:65,contrast:.4},reflectance:{roughness:.75,f0:.04}};
  const m={
    stone:new Mesh('house_masonry',[.66,.59,.46],'flat',detail?masonryMat:{}),
    trim:new Mesh('house_carved_stone',[.79,.72,.57],'smooth',detail?stoneMat:{}),
    roof:new Mesh('house_roof_shell',[.14,.18,.17],'smooth',detail?slateMat:{}),
    tiles:new Mesh('house_roof_tiles',[.22,.26,.24],'flat',detail?slateMat:{}),
    wood:new Mesh('house_door',[.17,.105,.047],'flat',detail?{pattern:{kind:'ridged',frequency:[15,1,3],low:[.5,.48,.4],high:[1,1,.9],bump:.006,warp:.18,octaves:2,seed:44,contrast:.35}}:{}),
    glass:new Mesh('house_glass',[.074,.101,.08],'flat',{reflectance:{roughness:.2,f0:.12}}),
    branches:new Mesh('house_tree_tracery',[.49,.33,.12],'smooth',{reflectance:{roughness:.58,f0:.07}}),
    paving:new Mesh('house_paving',[.5,.47,.38],'flat',detail?stoneMat:{}),
    leaves:new Mesh('house_leaves',[.105,.22,.065],'flat'),
    flowers:new Mesh('house_flowers',[.86,.8,.64],'flat'),
    violet:new Mesh('house_violet',[.33,.2,.46],'flat'),
    lamp:new Mesh('house_lantern',[.73,.47,.13],'smooth'),
  };
  const s=m.stone,t=m.trim, y=p.floorY;
  const motifs=new Map(),motifInstances=[];
  const repeatMotif=(kind,w,h,position,angle,thin=false)=>{
    const id='house_motif_'+kind;
    if(!motifs.has(id)){
      const mesh=new Mesh(id,m.branches.color,'smooth',m.branches.material);
      tree(mesh,w,h,(x,v,d=0)=>[x,v,d],thin);motifs.set(id,mesh);
    }
    motifInstances.push({id:kind+'_motif_'+motifInstances.length,asset:id,position,rotation:[0,Math.sin(angle/2),0,Math.cos(angle/2)],scale:1,color:m.branches.color});
  };
  const mapFront=(cx,z,base=0)=>(u,v,d=0)=>[cx+u,base+v,z+d];
  const mapSide=(z,x,base=0)=>(u,v,d=0)=>[x-d,base+v,z+u];
  // Shared elevated slab; the bay projection receives its own polygonal base.
  const slabBack=p.backZ-.15,slabFront=p.frontZ+.33,slabMid=(slabBack+slabFront)/2,slabDepth=slabFront-slabBack;
  s.box([1.8,y/2,slabMid],[7.95,y,slabDepth]);
  s.box([1.8,y-.04,slabMid],[8.14,.16,slabDepth+.15]);
  const bays=8,theta=Array.from({length:bays+1},(_,i)=>-Math.PI/2+i*Math.PI/bays);
  const bay=(a,r=p.bayRadius,yy=0)=>[p.bayX+r*Math.sin(a),yy,p.bayZ+r*Math.cos(a)];
  for(let i=0;i<bays;i++){
    const a=bay(theta[i],p.bayRadius+.12),b=bay(theta[i+1],p.bayRadius+.12);
    s.quad([a[0],0,a[2]],[b[0],0,b[2]],[b[0],1.25,b[2]],[a[0],1.25,a[2]]);
    s.quad([p.bayX,1.25,p.bayZ],[a[0],1.25,a[2]],[b[0],1.25,b[2]],[p.bayX,1.25,p.bayZ]);
  }
  // Three physical stair risers, leading to the open entrance.
  for(let i=0;i<3;i++){
    const depth=1.13-i*.27;
    s.box([-.6,(i+1)*y/6,p.frontZ+depth/2],[3.25,(i+1)*y/3,depth]);
  }
  // Recessed enclosed volume; lateral windows are separate openings in strips.
  const wallBack=p.backZ+.2,wallMid=(wallBack+1.8)/2,wallDepth=1.8-wallBack;
  s.box([2.85,1.03,wallMid],[5.8,.82,wallDepth]);
  s.box([2.85,3.55,wallMid],[5.8,.5,wallDepth]);
  s.box([5.67,2.4,wallMid],[.18,2.15,wallDepth]);
  s.box([2.85,2.4,p.backZ+.35],[5.8,2.15,.18]);
  for(const z of [-2.65,-.85,.95,1.85])s.box([0,2.4,z],[.18,2.15,.25]);
  for(const z of [-1.78,.05]) {
    const map=mapSide(z,-.025,1.35),top=ogive(1.45,2.03,1.28,8);
    m.glass.panel([[-.725,0],...top,[.725,0]],map,.035);
    border(t,1.55,2.1,1.34,map,.067,8);
    if(detail)tree(m.branches,1.31,1.9,mapSide(z,-.12,1.38));
  }
  // Door surround is behind the porch entrance, not on its outer arch plane.
  const doorX=-.4,doorZ=1.96;
  s.box([doorX-1.02,2.25,doorZ],[.65,3.25,.22]);
  s.box([doorX+1.28,2.25,doorZ],[1.1,3.25,.22]);
  s.box([doorX,3.57,doorZ],[1.45,.55,.22]);
  const doorTop=ogive(1.36,2.65,1.72,12);
  m.wood.panel([[-.68,0],...doorTop,[.68,0]],mapFront(doorX,doorZ+.04,y),.09);
  border(t,1.55,2.82,1.82,mapFront(doorX,doorZ+.13,y),.09,12);
  if(detail){
    border(m.branches,1.23,2.53,1.65,mapFront(doorX,doorZ+.2,y+.04),.035,10);
    tree(m.branches,1.12,2.39,mapFront(doorX,doorZ+.2,y+.05));
    m.branches.tube([[doorX,y+.05,doorZ+.2],[doorX,y+2.48,doorZ+.2]],.016,5);
  }
  // Open porch: columns are native plants, one shared definition and four instances.
  const columnZ=Array.from({length:4},(_,i)=>p.backZ+.18+(p.frontZ-p.backZ-.18)*i/3);
  const eaveWarp=z=>.4-.9*Math.max(0,Math.min(1,(z-(p.backZ-.35))/(p.frontZ+.25-(p.backZ-.35))));
  const columnTop=z=>p.eaveY+eaveWarp(z)-.23;
  const topY=p.eaveY-.23, colHeight=topY-y;
  for(let k=0;k<columnZ.length-1;k++) {
    const za=columnZ[k],zb=columnZ[k+1],mid=(za+zb)/2,w=zb-za-.27;
    const map=mapSide(mid,p.porchX,y+1.02);
    const localTop=columnTop(mid),top=border(t,w,localTop-y-1.02,.98,map,.082,10);
    if(detail)repeatMotif('porch',w*.87,1.48,[p.porchX-.035,y+1.05,mid],-Math.PI/2);
    s.box([p.porchX,y+.15,mid],[.34,.26,zb-za]);
    s.box([p.porchX,y+1.0,mid],[.36,.15,zb-za]);
    t.box([p.porchX,y+1.08,mid],[.4,.065,zb-za+.04]);
    for(let j=0;j<3;j++) {
      const zc=za+(j+.5)*(zb-za)/3,ww=(zb-za)/3-.08,small=mapSide(zc,p.porchX,y+.28);
      border(t,ww,.68,.32,small,.04,6);
      if(detail)repeatMotif('railing',ww*.82,.62,[p.porchX-.04,y+.3,zc],-Math.PI/2,true);
    }
    // Upper spandrel is a real surface above the curved opening.
    for(let j=0;j<top.length-1;j++){
      const a=top[j],b=top[j+1];
      s.quad(map(a[0],a[1],0),map(b[0],b[1],0),map(b[0],localTop-y-1.02+.13,0),map(a[0],localTop-y-1.02+.13,0));
    }
  }
  // Entrance arch spans from porch corner to the bay-side pier.
  const entryCenter=-.73,entryWidth=2.55;
  border(t,entryWidth,3.08,1.96,mapFront(entryCenter,p.frontZ+.025,y),.13,14);
  if(detail){
    border(m.branches,entryWidth-.18,2.99,1.93,mapFront(entryCenter,p.frontZ+.09,y+.04),.025,12);
    const right=entryCenter+entryWidth/2;
    for(const xx of [right]){
      s.box([xx,y+1.65,p.frontZ],[.28,3.3,.35]);
      t.box([xx,y+.12,p.frontZ],[.5,.24,.5]);
    }
  }
  // Faceted bay: four front faces, separate glass, stone piers and pointed frames.
  const faces=4;
  for(let k=0;k<faces;k++) {
    const a=-Math.PI/2+k*Math.PI/faces,b=a+Math.PI/faces,aa=bay(a),bb=bay(b),mid=lerp(aa,bb,.5);
    const u=unit(sub(bb,aa)),normal=[(aa[0]+bb[0])/2-p.bayX,0,(aa[2]+bb[2])/2-p.bayZ];
    const nn=unit(normal),width=Math.hypot(...sub(bb,aa));
    const map=(x,v,d=0)=>add([mid[0],1.25+v,mid[2]],add(mul(u,x),mul(nn,d)));
    const w=width-.18,h=2.16,top=ogive(w,h,1.36,10);
    m.glass.panel([[-w/2,0],...top,[w/2,0]],map,.025);
    border(t,w,h,1.36,map,.07,10);
    for(const yy of [1.2,3.57,3.7])t.tube([[aa[0],yy,aa[2]],[bb[0],yy,bb[2]]],yy===3.7?.08:.065,6);
    for(const point of [aa]){
      s.box([point[0],2.37,point[2]],[.17,2.36,.17]);
      t.box([point[0],1.3,point[2]],[.29,.16,.29]);
      t.box([point[0],3.5,point[2]],[.26,.14,.26]);
    }
    if(detail)repeatMotif('bay',w*.9,h*.94,map(0,.04,.055),Math.atan2(nn[0],nn[2]));
  }
  // Curved asymmetric main roof, sampled once and used by shells, trim and tiles.
  const ry=x=>x<=p.ridgeX?p.eaveY+(p.ridgeY-p.eaveY)*((x-p.roofLeft)/(p.ridgeX-p.roofLeft))**2.05:p.eaveY+(p.ridgeY-p.eaveY)*((p.roofRight-x)/(p.roofRight-p.ridgeX))**1.6;
  const roofZ0=p.backZ-.35,roofZ1=p.frontZ+.25;
  const roofPoint=(x,z,lift=0)=>{
    const along=(z-roofZ0)/(roofZ1-roofZ0),blend=x<=p.ridgeX?(x-p.roofLeft)/(p.ridgeX-p.roofLeft):(p.roofRight-x)/(p.roofRight-p.ridgeX);
    const edge=x<=p.ridgeX?eaveWarp(z):-.1;
    return [x,ry(x)+edge*(1-blend)+.4*along*blend+.075*Math.sin(Math.PI*along)+lift,z];
  };
  for(const [x0,x1] of [[p.roofLeft,p.ridgeX],[p.ridgeX,p.roofRight]]) {
    const nx=20,nz=5,first=m.roof.positions.length;
    for(let i=0;i<=nx;i++)for(let j=0;j<=nz;j++)m.roof.v(roofPoint(x0+(x1-x0)*i/nx,roofZ0+(roofZ1-roofZ0)*j/nz));
    for(let i=0;i<nx;i++)for(let j=0;j<nz;j++){
      const a=first+i*(nz+1)+j,b=a+nz+1;
      m.roof.tri(a,a+1,b+1);m.roof.tri(a,b+1,b);
    }
  }
  for(const z of [roofZ0,roofZ1]) {
    const profile=[];
    for(let i=0;i<=48;i++){const x=p.roofLeft+(p.roofRight-p.roofLeft)*i/48;profile.push(roofPoint(x,z,.035));}
    t.tube(profile,.092,6,[0,0,1]);
    if(detail)m.branches.tube(profile.map(q=>add(q,[0,.075,0])),.027,5,[0,0,1]);
  }
  for(const x of [p.roofLeft,p.roofRight,p.ridgeX])t.tube(Array.from({length:17},(_,j)=>roofPoint(x,roofZ0+(roofZ1-roofZ0)*j/16,-.015)),x===p.ridgeX?.085:.065,6,[1,0,0]);
  // Front gable infill; bay roof will occlude the lower right region naturally.
  const gx=[];for(let i=0;i<=32;i++){const x=p.roofLeft+.13+(p.roofRight-p.roofLeft-.26)*i/32;gx.push([x,Math.max(3.76,roofPoint(x,roofZ1)[1]-.1)]);}
  const gmap=(x,v,d=0)=>[x,v,roofZ1-.065+d];
  for(let i=0;i<gx.length-1;i++)m.glass.quad(gmap(gx[i][0],3.76),gmap(gx[i+1][0],3.76),gmap(...gx[i+1]),gmap(...gx[i]));
  if(detail) {
    const gm=(u,v,d=0)=>[p.ridgeX+u,3.92+v,roofZ1+.035+d];
    tree(t,2.32,2.82,gm);
    for(const sign of [-1,1])t.tube(curve(gm(0,.05),gm(sign*.38,.15),gm(sign*1.5,.5),gm(sign*2.0,.19),10),.072,6,[0,0,1]);
    m.branches.tube([[p.ridgeX,p.ridgeY+.28,roofZ1],[p.ridgeX,p.ridgeY+.94,roofZ1]],.052,6,[0,0,1],.012);
  }
  // Bay roof is a curved radial sector, connected visually to the main roof.
  const capPoint=(a,q,lift=0)=>{
    const r=(p.bayRadius+.24)*q;
    return [p.bayX+p.bayApexOffsetX*(1-q)+r*Math.sin(a),p.bayEaveY+(p.bayApexY-p.bayEaveY)*(1-q)**1.3+lift,p.bayZ+r*Math.cos(a)];
  };
  const rings=8,angular=32,first=m.roof.positions.length;
  for(let i=0;i<=rings;i++)for(let j=0;j<=angular;j++)m.roof.v(capPoint(-Math.PI/2+j*Math.PI/angular,.025+.975*i/rings));
  for(let i=0;i<rings;i++)for(let j=0;j<angular;j++){
    const a=first+i*(angular+1)+j,b=a+angular+1;m.roof.tri(a,b,a+1);m.roof.tri(a+1,b,b+1);
  }
  t.tube(Array.from({length:33},(_,i)=>capPoint(-Math.PI/2+i*Math.PI/32,1,.015)),.095,6,[0,1,0]);
  if(detail) {
    // Separate chamfered shingle faces, deterministic stagger and overlap.
    for(const [x0,x1] of [[p.roofLeft,p.ridgeX],[p.roofRight,p.ridgeX]]) {
      const rows=Math.ceil(Math.abs(x1-x0)/.31),cols=21;
      for(let i=0;i<rows;i++)for(let j=0;j<cols;j++){
        const a=i/rows,b=Math.min(1,(i+1.24)/rows),za=roofZ0+(j+(i%2)*.5)*(roofZ1-roofZ0)/cols;
        const zb=Math.min(roofZ1,za+(roofZ1-roofZ0)/cols*.96);if(za>=roofZ1)continue;
        const xa=x0+(x1-x0)*a,xb=x0+(x1-x0)*b;
        const mx=(xa+xb)/2,mz=(za+zb)/2,center=roofPoint(mx,mz),dx=(roofPoint(xb,mz)[1]-roofPoint(xa,mz)[1])/(xb-xa),dz=(roofPoint(mx,zb)[1]-roofPoint(mx,za)[1])/(zb-za);
        const poly=[[xa,za],[xb,za],[xb,zb],[xa,zb]].map(([x,z])=>[x,center[1]+dx*(x-mx)+dz*(z-mz)+.04+.035*(1-(x-xa)/(xb-xa))+.002*Math.sin(i*7+j*3),z]);
        poly.push(add(poly[0],[0,-.045,0]),add(poly[3],[0,-.045,0]));
        const start=m.tiles.positions.length;poly.forEach(v=>m.tiles.v(v));
        for(const face of [[0,2,1],[0,3,2],[0,4,5],[0,5,3]])m.tiles.tri(...face.map(q=>start+q));
      }
    }
    for(let i=0;i<7;i++){
      const q0=.98-i*.12,q1=Math.max(.09,q0-.16),cols=Math.max(7,Math.round(25*q0));
      for(let j=0;j<cols;j++){
        const a=-Math.PI/2+(j+(i%2)*.45)*Math.PI/cols,b=Math.min(Math.PI/2,a+Math.PI/cols*.95);if(a>=Math.PI/2)continue;
        const ma=(a+b)/2,mq=(q0+q1)/2,center=capPoint(ma,mq),du=mul(sub(capPoint(b,mq),capPoint(a,mq)),1/(b-a)),dv=mul(sub(capPoint(ma,q0),capPoint(ma,q1)),1/(q0-q1));
        const tilePoint=(angle,q)=>add(add(center,add(mul(du,angle-ma),mul(dv,q-mq))),[0,.04+.035*(q-q1)/(q0-q1),0]);
        const poly=[tilePoint(a,q0),tilePoint(a,q1),tilePoint(b,q1),tilePoint(b,q0)];
        poly.push(add(poly[0],[0,-.045,0]),add(poly[3],[0,-.045,0]));
        const start=m.tiles.positions.length;poly.forEach(v=>m.tiles.v(v));for(const face of [[0,2,1],[0,3,2],[0,4,5],[0,5,3]])m.tiles.tri(...face.map(q=>start+q));
      }
    }
  }
  // Chimney and plinth/cap; its base intersects the roof by construction.
  const chimneyX=p.chimneyX,chimneyZ=p.chimneyZ,chimneyBase=roofPoint(chimneyX,chimneyZ)[1]-.08;
  s.box([chimneyX,chimneyBase+p.chimneyHeight/2,chimneyZ],[.68,p.chimneyHeight,.7]);
  t.box([chimneyX,chimneyBase+p.chimneyHeight-.01,chimneyZ],[.85,.15,.87]);
  t.box([chimneyX,chimneyBase+p.chimneyHeight+.12,chimneyZ],[.92,.11,.94]);
  if(detail) {
    tree(m.branches,.5,1.3,mapSide(chimneyZ,chimneyX-.35,chimneyBase+.12),true);
    tree(m.branches,.5,1.3,mapFront(chimneyX,chimneyZ+.36,chimneyBase+.12),true);
    // Vases, wall lantern, door handle and paving.
    for(const [cx,cz,scale] of [[.9,2.6,1],[-1.4,-1.7,.65]])s.lathe(cx,cz,[[y,.23*scale],[y+.08,.27*scale],[y+.55*scale,.34*scale],[y+.73*scale,.25*scale],[y+.78*scale,.3*scale]],12);
    const lx=.72,lz=doorZ+.3,ly=2.8;
    m.branches.tube(curve([lx,ly+.45,doorZ],[lx,ly+.6,doorZ+.4],[lx,ly+.55,doorZ+.55],[lx,ly+.33,lz+.22],8),.023,6);
    m.lamp.lathe(lx,lz+.22,[[ly-.2,.12],[ly+.1,.12],[ly+.18,.17],[ly+.3,.02]],6);
    for(let j=0;j<4;j++){const a=j*Math.PI/2;m.branches.tube([[lx+.13*Math.cos(a),ly-.22,lz+.22+.13*Math.sin(a)],[lx+.13*Math.cos(a),ly+.18,lz+.22+.13*Math.sin(a)]],.014,4);}
    m.branches.lathe(doorX+.35,doorZ+.22,[[1.65,.035],[1.72,.035]],8);
    for(let i=0;i<11;i++)for(let j=0;j<9;j++){
      const xx=-3.5+i*.98+(j%2)*.25,zz=-4+j*.98;
      if(xx>-2.3&&xx<5.9&&zz>-3.5&&zz<2.85)continue;
      m.paving.box([xx+.09*Math.sin(i*7+j),-.015,zz+.08*Math.cos(i+j*9)],[.8+.12*Math.sin(i+j),.09,.72+.1*Math.cos(i*3+j)]);
    }
  }
  if(plants) {
    let seed=6189;const rnd=()=>{seed=(Math.imul(seed,1664525)+1013904223)>>>0;return seed/4294967296;};
    const clusters=[[-2.3,-3.7,.02],[-2.3,-.8,.02],[-2.3,1.7,.02],[2.2,3.5,.02],[5.4,3.2,.02],[6,2,.02],[.9,2.6,y+.76],[-1.4,-1.7,y+.48]];
    for(const [cx,cz,base] of clusters)for(let i=0;i<7;i++){
      const x=cx+(rnd()-.5)*.9,z=cz+(rnd()-.5)*.9,h=.18+rnd()*.58;
      m.leaves.tube([[x,base,z],[x+.05,base+h,z+.03]],.009,3,null,.004);
      for(let j=0;j<4;j++){
        const yy=base+h*(.2+j*.2),a=rnd()*Math.PI*2,L=.14+rnd()*.16;
        const c=[x,yy,z],end=[x+L*Math.cos(a),yy+.05,z+L*Math.sin(a)],side=[-Math.sin(a)*L*.32,0,Math.cos(a)*L*.32];
        m.leaves.quad(c,add(lerp(c,end,.5),side),end,sub(lerp(c,end,.5),side));
      }
      if(i%3===0){const f=i%2?m.flowers:m.violet;for(let k=0;k<5;k++){
        const a=k*Math.PI*.4,cc=[x,base+h,z],end=[x+.06*Math.cos(a),base+h+.025,z+.06*Math.sin(a)],side=[-.025*Math.sin(a),0,.025*Math.cos(a)];
        f.quad(cc,add(lerp(cc,end,.6),side),end,sub(lerp(cc,end,.6),side));
      }}
    }
    // Ivy grows along selected supports; not a random blanket over all masonry.
    for(const z0 of [columnZ[0],columnZ[3]])for(let i=0;i<20;i++){
      const yy=y+i*.13,zz=z0+Math.sin(i*.9)*.1,xx=p.porchX-.2;
      m.leaves.quad([xx,yy,zz],[xx-.065,yy+.035,zz-.05],[xx-.045,yy+.095,zz],[xx-.04,yy+.035,zz+.05]);
    }
  }
  const profile=[[0,1],[.07,1],[.09,.81],[.13,.81],[.16,.55],[.84,.48],[.87,.62],[.9,.62],[.93,.86],[1,.86]];
  const columnDefinition={id:'house_porch_column',profile:{id:'column_envelope',width:.48,height:colHeight},depth:.48,segments:[1,2,1],sections:{id:'column_profile',cross_section:'rectangle',interpolation:'linear',stations:profile.map(([at,width],i)=>({id:'level_'+i,at,width,depth:width}))}};
  const meshes=Object.values(m).concat([...motifs.values()]).filter(x=>x.triangles.length);
  const instances=Object.values(m).filter(x=>x.triangles.length).map(x=>({id:x.id,asset:x.id,position:[0,0,0],rotation:[0,0,0,1],scale:1,color:x.color}));
  instances.push(...motifInstances);
  for(let i=0;i<columnZ.length;i++){
    const height=columnTop(columnZ[i])-y;
    instances.push({id:'porch_support_'+i,asset:columnDefinition.id,position:[p.porchX,y+height/2,columnZ[i]],rotation:[0,0,0,1],scale:height/colHeight,color:[.76,.69,.55]});
  }
  for(const mesh of meshes)for(const point of mesh.positions){point[0]*=p.horizontalScale;point[2]*=p.horizontalScale;}
  for(const inst of instances){inst.position[0]*=p.horizontalScale;inst.position[2]*=p.horizontalScale;}
  columnDefinition.profile.width*=p.horizontalScale;columnDefinition.depth*=p.horizontalScale;
  return {parameters:p,meshes,instances,columnDefinition,columnMaterial:detail?stoneMat:{},light:{direction:[.5,-1,-.6],ambient:.48,shadow:{center:[1.5,2,0],extent:10,resolution:2048,bias:.0008}},correspondence:{implantation:['house_masonry','house_paving'],porch:['house_porch_column','porch_support_0','porch_support_1','porch_support_2','porch_support_3','house_carved_stone'],entrance:['house_door','house_carved_stone'],bay:['house_masonry','house_glass','house_carved_stone'],coverings:['house_roof_shell','house_roof_tiles','house_glass'],motifs:['house_tree_tracery','house_motif_porch','house_motif_railing','house_motif_bay'],accessories:['house_lantern','house_leaves','house_flowers','house_violet']}};
}

export async function publishHouse(eng,house,{clean=false}={}) {
  const edits=[];
  const current=await eng({op:'inspect_meshes'});
  const desiredAssets=new Set([...house.meshes.map(x=>x.id),house.columnDefinition.id]);
  const desiredInstances=new Set(house.instances.map(x=>x.id));
  for(const inst of current.instances)if(!desiredInstances.has(inst.id))edits.push({op:'mesh',edit:{kind:'remove_instance',id:inst.id}});
  for(const asset of current.assets)if(!desiredAssets.has(asset.id))edits.push({op:'mesh',edit:{kind:'remove_asset',id:asset.id}});
  if(clean){
    const state=await eng({op:'inspect'});
    for(const e of state.scene.entities)if(e.id!=='player')edits.push({op:'remove',id:e.id});
    const player=state.scene.entities.find(x=>x.id==='player');
    edits.push({op:'upsert',entity:{...player,visible:false,transform:{...player.transform,position:[0,-100,0]}}});
  }
  edits.push({op:'upsert',entity:{id:'study_ground',name:'studio ground',transform:{position:[0,-.14,0],size:[200,.2,200]},color:[.52,.48,.41]}});
  edits.push({op:'upsert',entity:{id:'study_backdrop',name:'studio backdrop',transform:{position:[0,35,-14],size:[200,70,.1]},color:[.62,.57,.49]}});
  edits.push({op:'plant',edit:{kind:'put',definition:house.columnDefinition,detail_policy:'physical',animation_translation_policy:'physical'}});
  edits.push({op:'mesh',edit:{kind:'set_material',asset:house.columnDefinition.id,material:house.columnMaterial}});
  for(const mesh of house.meshes)edits.push({op:'mesh',edit:{kind:'put_asset',asset:mesh.asset()}});
  for(const instance of house.instances)edits.push({op:'mesh',edit:{kind:'put_instance',instance}});
  edits.push({op:'set_camera',camera:house.parameters.camera},{op:'set_light',light:house.light});
  const preview=await eng({op:'preview_scene_edits',edits});
  const result=await eng({op:'edit_scene',edits});
  return {preview,result,counts:house.meshes.map(x=>({id:x.id,vertices:x.positions.length,triangles:x.triangles.length}))};
}

// Minimal standard-library MCP transport for replaying this authoring experiment.
// Each run owns a new headless process; it never attaches to an existing demo.
export async function runStudy({binary,root,reference}) {
  const fs=await import('node:fs/promises'), cp=await import('node:child_process'), path=await import('node:path'),crypto=await import('node:crypto');
  await fs.mkdir(path.join(root,'artifacts'),{recursive:true});
  if(reference)await fs.copyFile(reference,path.join(root,'artifacts/reference.png'));
  const proc=cp.spawn(binary,['--mcp','--root',root],{stdio:['pipe','pipe','pipe']});
  let seq=0,buffer='',stderr='',revision=0;
  const pending=new Map(),audit=[];
  const fail=message=>{for(const task of pending.values()){clearTimeout(task.timer);task.reject(new Error(message));}pending.clear();};
  proc.stderr.on('data',x=>{stderr=(stderr+x).slice(-8000);});
  proc.on('error',e=>fail(e.message));
  proc.on('exit',code=>fail('MCP exited '+code+' '+stderr));
  proc.stdout.on('data',bytes=>{
    buffer+=bytes;
    const lines=buffer.split('\n');buffer=lines.pop();
    for(const line of lines)if(line.trim()){
      let message;try{message=JSON.parse(line);}catch{fail('Invalid MCP JSON');continue;}
      const task=pending.get(message.id);if(!task)continue;
      pending.delete(message.id);clearTimeout(task.timer);
      if(message.error)task.reject(new Error(JSON.stringify(message.error)));else task.resolve(message.result);
    }
  });
  const rpc=(method,params)=>new Promise((resolve,reject)=>{
    const id=++seq,timer=setTimeout(()=>{pending.delete(id);reject(new Error('MCP timeout: '+method+' '+stderr));},30000);
    pending.set(id,{resolve,reject,timer});proc.stdin.write(JSON.stringify({jsonrpc:'2.0',id,method,params})+'\n');
  });
  const edits=new Set(['edit_scene','edit_mesh','edit_plant','set_camera','set_light','undo','redo','load_scene']);
  const eng=async command=>{
    const args={command};if(edits.has(command.op))args.expected_revision=revision;
    const response=await rpc('tools/call',{name:'engine_command',arguments:args});
    const text=response.content?.find(x=>x.type==='text')?.text;
    let result;try{result=JSON.parse(text);}catch{throw Error('Expected structured engine response: '+text);}
    audit.push({op:command.op,expected_revision:args.expected_revision,error:response.isError??false,revision:result.revision,path:result.path});
    if(response.isError)throw Error(JSON.stringify(result));
    if((edits.has(command.op)||command.op==='inspect')&&result.revision!==undefined)revision=result.revision;
    return result;
  };
  const capture=async filename=>eng({op:'capture',path:'artifacts/'+filename,width:1536,height:1024});
  const equal=async filename=>(await eng({op:'compare_images',reference_path:'artifacts/reconstruction.png',candidate_path:'artifacts/'+filename})).report.metrics.exact_rgb_match;
  try {
    const initialization=await rpc('initialize',{protocolVersion:'2024-11-05',capabilities:{},clientInfo:{name:'reference-study',version:'1'}});
    proc.stdin.write(JSON.stringify({jsonrpc:'2.0',method:'notifications/initialized'})+'\n');
    const tools=await rpc('tools/list');
    if(!tools.tools.some(t=>t.name==='engine_command'))throw Error('Ignis engine_command unavailable');
    await fs.writeFile(path.join(root,'mcp-tools.json'),JSON.stringify(tools,null,2));
    const initial=await eng({op:'inspect'}),house=buildHouse({detail:true,vegetation:true});
    await publishHouse(eng,house,{clean:true});
    const finalCapture=await capture('reconstruction.png');
    await eng({op:'save_scene',path:'examples/house.scene.json'});
    await eng({op:'save_replay',path:'artifacts/house.replay.json'});
    await capture('repeated.png');
    const validation={repeat_exact:await equal('repeated.png')};
    const replay=await eng({op:'run_replay',path:'artifacts/house.replay.json'});
    validation.replay_matched=replay.matched;validation.replay_ticks=replay.ticks;
    await eng({op:'capture_replay',replay_path:'artifacts/house.replay.json',path:'artifacts/replay.png',width:1536,height:1024});
    validation.replay_pixels_exact=await equal('replay.png');
    const state=await eng({op:'inspect'});
    const clayEdits=[...state.scene.meshes.assets.map(a=>({op:'mesh',edit:{kind:'set_material',asset:a.id,material:{}}})),...state.scene.meshes.instances.map(i=>({op:'mesh',edit:{kind:'put_instance',instance:{...i,color:[.62,.62,.62]}}}))];
    await eng({op:'edit_scene',edits:clayEdits});await capture('clay.png');await eng({op:'undo'});
    await capture('restored.png');validation.undo_exact=await equal('restored.png');
    const before=await eng({op:'inspect'});let rejected=false;
    try{await eng({op:'edit_mesh',edit:{kind:'put_asset',asset:{id:'house_budget_probe',geometry:{positions:[[0,0,0],[.1,0,0],[0,.1,0]],triangles:[[0,1,2]]}}}});}catch(e){rejected=e.message.includes('Mesh asset/instance budget exceeded');validation.limit_error=e.message;}
    const after=await eng({op:'inspect'});
    validation.limit_error_rejected=rejected;
    validation.limit_preserves_scene=JSON.stringify(before.scene)===JSON.stringify(after.scene)&&before.revision===after.revision;
    await eng({op:'set_camera',camera:{position:[-11,10,14],target:[1.7,2.6,-.3],fov_degrees:34}});await capture('spatial.png');
    await eng({op:'load_scene',path:'examples/house.scene.json'});await capture('reloaded.png');validation.load_exact=await equal('reloaded.png');
    const meshInfo=await eng({op:'inspect_meshes'});
    const validIds=new Set([...meshInfo.assets.map(x=>x.id),...meshInfo.instances.map(x=>x.id)]);
    validation.correspondence_resolves=Object.values(house.correspondence).flat().every(id=>validIds.has(id));
    const comparison=await eng({op:'compare_images',reference_path:'artifacts/reference.png',candidate_path:'artifacts/reconstruction.png',regions:[{id:'roof',x:80,y:65,width:1400,height:415},{id:'porch',x:100,y:410,width:620,height:450},{id:'entry',x:670,y:410,width:350,height:490},{id:'bay',x:990,y:440,width:485,height:425}],difference_path:'artifacts/difference.png'});
    const sourceBytes=await fs.readFile(path.join(root,'artifacts/reference.png'));
    const source={path:'artifacts/reference.png',sha256:crypto.createHash('sha256').update(sourceBytes).digest('hex'),width:sourceBytes.readUInt32BE(16),height:sourceBytes.readUInt32BE(20)};
    const report={scope:'First reconstruction from skill, image and public MCP; not an independent blind trial or accepted faithful model',source,initialization,build_id:initial.build_id,parameters:house.parameters,correspondence:house.correspondence,counts:{assets:meshInfo.assets.length,instances:meshInfo.instances.length,vertices:meshInfo.assets.reduce((n,a)=>n+a.vertices,0),triangles:meshInfo.assets.reduce((n,a)=>n+a.triangles,0),draw_triangles:finalCapture.mesh_triangle_count},limits:meshInfo.limits,validation,comparison:comparison.report,audit};
    await fs.writeFile(path.join(root,'report.json'),JSON.stringify(report,null,2)+'\n');
    const rows=[{id:'total',metrics:report.comparison.metrics},...report.comparison.regions.map(x=>({id:x.region.id,metrics:x.metrics}))].map(x=>`<tr><td>${x.id}</td><td>${x.metrics.rgb_mae.toFixed(4)}</td><td>${x.metrics.ssim_luma.toFixed(4)}</td></tr>`).join('');
    const counts=`${report.counts.assets} assets · ${report.counts.instances} instâncias · ${report.counts.vertices} vértices armazenados · ${report.counts.triangles} triângulos armazenados · ${report.counts.draw_triangles} triângulos desenhados.`;
    const template=await fs.readFile(new URL('./house_skill_report.html',import.meta.url),'utf8');
    await fs.writeFile(path.join(root,'index.html'),template.replace('__METRIC_ROWS__',rows).replace('__COUNTS__',counts));
    if(Object.entries(validation).some(([k,v])=>k!=='limit_error'&&k!=='replay_ticks'&&v!==true))throw Error('Validation failed: '+JSON.stringify(validation));
    return report;
  } finally {
    proc.stdin.end();
    await new Promise(resolve=>{if(proc.exitCode!==null){resolve();return;}const timer=setTimeout(()=>{proc.kill('SIGTERM');resolve();},2000);proc.once('exit',()=>{clearTimeout(timer);resolve();});});
  }
}
