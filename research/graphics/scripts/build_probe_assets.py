"""Headless Blender source assets and restricted indexed export for the appearance lab."""
import bpy, bmesh, json, math, random, struct
from pathlib import Path
from mathutils import Vector, noise

ROOT = Path(__file__).resolve().parents[1] / 'artifacts/appearance-probe/assets'
MATERIALS = [
    ('stone', (0.29,0.26,0.22), .88, 0.),
    ('wood', (.30,.105,.027), .66, 0.),
    ('iron', (.26,.29,.33), .28, 1.),
    ('brass', (.65,.39,.10), .18, 1.),
    ('paint', (.018,.16,.22), .24, 0.),
    ('bark', (.12,.055,.022), .92, 0.),
    ('leaf', (.09,.24,.022), .57, 0.),
]

def material_setup():
    result=[]
    for name,color,rough,metal in MATERIALS:
        m=bpy.data.materials.new(name)
        m.diffuse_color=(*color,1)
        bs=m.node_tree.nodes.get('Principled BSDF')
        bs.inputs['Base Color'].default_value=(*color,1)
        bs.inputs['Roughness'].default_value=rough
        bs.inputs['Metallic'].default_value=metal
        result.append(m)
    return result

def rock():
    bm=bmesh.new(); bmesh.ops.create_icosphere(bm, subdivisions=7, radius=1.)
    for v in bm.verts:
        p=v.co.copy()
        macro=noise.noise_vector(p*2.1)[0]
        fine=noise.noise_vector(p*19.7)[1]
        micro=noise.noise_vector(p*73.1)[2]
        r=1+.23*macro+.045*fine+.012*micro
        v.co=Vector((p.x*1.13*r,p.y*.87*r,p.z*.77*r))
        v.co.z=max(v.co.z,-.57+.025*fine)
    bm.normal_update(); mesh=bpy.data.meshes.new('weathered-rock'); bm.to_mesh(mesh); bm.free()
    for p in mesh.polygons: p.use_smooth=True
    return mesh

class Builder:
    def __init__(self): self.v=[]; self.f=[]; self.m=[]; self.s=[]
    def add(self, verts, faces, mat, smooth=False):
        base=len(self.v); self.v.extend(verts)
        self.f.extend([tuple(base+i for i in f) for f in faces]); self.m.extend([mat]*len(faces)); self.s.extend([smooth]*len(faces))
    def tube(self,a,b,r0,r1,mat,sides=12,steps=6):
        a,b=Vector(a),Vector(b); axis=(b-a).normalized()
        u=axis.cross(Vector((0,0,1)))
        if u.length<.01: u=axis.cross(Vector((0,1,0)))
        u.normalize(); v=axis.cross(u).normalized(); vs=[]
        for j in range(steps+1):
            t=j/steps; center=a.lerp(b,t)
            for i in range(sides):
                ang=i*2*math.pi/sides
                radius=(r0*(1-t)+r1*t)*(1+.065*math.sin(i*4.7+j*1.2))
                vs.append(center+(u*math.cos(ang)+v*math.sin(ang))*radius)
        fs=[]
        for j in range(steps):
            for i in range(sides):
                k=j*sides+i; kn=j*sides+(i+1)%sides
                fs.append((k,kn,kn+sides,k+sides))
        fs.extend([tuple(reversed(range(sides))),tuple(steps*sides+i for i in range(sides))])
        self.add(vs,fs,mat,True)
    def box(self,center,scale,mat):
        bm=bmesh.new(); bmesh.ops.create_cube(bm,size=2)
        for v in bm.verts: v.co=Vector(tuple(v.co[i]*scale[i] for i in range(3)))
        bmesh.ops.bevel(bm,geom=list(bm.edges),offset=min(scale)*.15,segments=3,affect='EDGES')
        # Subdivide planar panels to carry visible wood grain in vertex color.
        bmesh.ops.triangulate(bm,faces=list(bm.faces))
        bm.verts.ensure_lookup_table(); bm.verts.index_update()
        self.add([v.co+Vector(center) for v in bm.verts],[tuple(v.index for v in f.verts) for f in bm.faces],mat)
        bm.free()
    def mesh(self,name):
        m=bpy.data.meshes.new(name); m.from_pydata(self.v,[],self.f); m.update()
        for p,mat,smooth in zip(m.polygons,self.m,self.s): p.material_index=mat; p.use_smooth=smooth
        return m

def chest():
    b=Builder()
    # Separate boards, recessed joints and rounded fittings; useful as a cheap geometry control.
    for x in [-.8,-.4,0,.4,.8]:
        b.box((x,0,.70),(.192,.59,.07),1)
        for y in [-.60,.60]: b.box((x,y,.30),(.192,.055,.32),1)
    for x in [-.98,.98]:
        for y in [-.4,0,.4]: b.box((x,y,.30),(.055,.19,.32),1)
    for x in [-.72,.72]:
        b.box((x,0,.79),(.055,.66,.025),2)
        for y in [-.66,.66]:
            b.box((x,y,.35),(.055,.025,.40),2)
            for z in [.1,.4,.65]:
                b.tube((x,y,z),(x,y+math.copysign(.035,y),z),.028,.028,3,12,1)
    b.box((0,-.685,.52),(.12,.035,.16),3)
    b.box((0,-.73,.55),(.052,.015,.07),2)
    b.box((0,0,.805),(.48,.28,.014),4)
    for x in [-1.055,1.055]:
        b.tube((x,-.22,.4),(x,.22,.4),.028,.028,2,16,2)
    return b.mesh('wood-iron-brass-painted-chest')

def tree():
    rng=random.Random(7231); b=Builder()
    b.tube((0,0,-1.2),(.08,.025,.95),.115,.035,5,20,28)
    for k in range(26):
        a=k*2.39996; z=-.55+k*.052
        start=Vector((.04,0,z)); reach=.55+.26*math.sin(k*.71)**2
        end=Vector((math.cos(a)*reach,math.sin(a)*reach,z+.6))
        b.tube(start,end,.044,.008,5,10,8)
        for j in range(4):
            t=.38+j*.18; root=start.lerp(end,t)
            tip=root+Vector((math.cos(a+j+1)*.27,math.sin(a+j+1)*.27,.21))
            b.tube(root,tip,.012,.003,5,7,3)
            for _ in range(22):
                c=root.lerp(tip,rng.random())+Vector((rng.uniform(-.17,.17),rng.uniform(-.17,.17),rng.uniform(-.1,.19)))
                yaw=rng.random()*math.tau; length=rng.uniform(.07,.12)
                u=Vector((math.cos(yaw),math.sin(yaw),rng.uniform(-.5,.5))).normalized()*length
                v=Vector((-math.sin(yaw),math.cos(yaw),rng.uniform(-.2,.2)))*length*.42
                # Folded leaf blade, explicit outline instead of alpha-texture cards.
                verts=[c-u,c-v,c+Vector((0,0,.012)),c+v,c+u]
                b.add(verts,[(0,1,2),(0,2,3),(1,4,2),(2,4,3)],6)
    return b.mesh('branching-tree-with-individual-leaves')

def color_at(p,mat):
    base=MATERIALS[mat][1]
    n=noise.noise_vector(p*15.3)[0]
    f=.86+.20*n
    if mat==0:
        f=.72+.36*noise.noise_vector(p*7.1)[1]+.10*math.sin(p.z*38+p.x*7)
    elif mat==1: f=.72+.23*math.sin(p.x*39+noise.noise_vector(p*3)[0]*5)
    elif mat==6: f=.68+.40*noise.noise_vector(p*31)[2]
    return tuple(max(.002,min(.95,c*f)) for c in base)

def export(name,mesh,mats,offset):
    assert not mesh.validate(), 'invalid topology'
    mesh.update(); mesh.calc_loop_triangles()
    points=[v.co.copy() for v in mesh.vertices]
    lo=Vector(tuple(min(p[i] for p in points) for i in range(3)))
    hi=Vector(tuple(max(p[i] for p in points) for i in range(3)))
    center=(lo+hi)*.5; radius=max((p-center).length for p in points); scale=1.22/radius
    for v in mesh.vertices: v.co=(v.co-center)*scale
    mesh.update(); mesh.calc_loop_triangles()
    colors=mesh.color_attributes.new(name='surface_color',type='FLOAT_COLOR',domain='CORNER')
    for m in mats: mesh.materials.append(m)
    for poly in mesh.polygons:
        for li in poly.loop_indices:
            p=mesh.vertices[mesh.loops[li].vertex_index].co
            colors.data[li].color=(*color_at(p,poly.material_index),1)
    data=bytearray(); indices=[]; unique={}; degenerate=0
    for tri in mesh.loop_triangles:
        ps=[mesh.vertices[i].co for i in tri.vertices]
        if (ps[1]-ps[0]).cross(ps[2]-ps[0]).length<1e-10:
            degenerate+=1; continue
        for li in tri.loops:
            loop=mesh.loops[li]; p=mesh.vertices[loop.vertex_index].co
            n=mesh.corner_normals[li].vector
            # Blender Z up to engine Y up; right-handed rotation.
            c=colors.data[li].color
            packed=struct.pack('<9fI',p.x,p.z+.12,-p.y,n.x,n.z,-n.y,*c[:3],tri.material_index)
            if packed not in unique: unique[packed]=len(unique); data.extend(packed)
            indices.append(unique[packed])
    path=ROOT/name; path.mkdir(parents=True,exist_ok=True)
    (path/'vertices.bin').write_bytes(data)
    (path/'indices.bin').write_bytes(struct.pack('<%dI'%len(indices),*indices))
    obj=bpy.data.objects.new(name,mesh); bpy.context.collection.objects.link(obj); obj.location.x=offset
    stats={'schema':1,'name':name,'blender':bpy.app.version_string,'seed':7231,'triangles':len(indices)//3,'vertices':len(unique),'vertex_bytes':len(data),'index_bytes':len(indices)*4,'discarded_degenerate':degenerate,'materials':sorted(set(p.material_index for p in mesh.polygons)),'bounds_engine':[[min(struct.unpack_from('<3f',data,i)[j] for i in range(0,len(data),40)) for j in range(3)],[max(struct.unpack_from('<3f',data,i)[j] for i in range(0,len(data),40)) for j in range(3)]]}
    (path/'asset.json').write_text(json.dumps(stats,indent=2))
    return obj,stats

def review(objects):
    scene=bpy.context.scene; scene.render.engine='BLENDER_WORKBENCH'
    scene.render.resolution_x=1100; scene.render.resolution_y=460; scene.render.resolution_percentage=100
    scene.display.shading.light='STUDIO'; scene.display.shading.color_type='MATERIAL'
    scene.display.shading.show_shadows=True; scene.display.shading.show_cavity=True
    scene.world.color=(.08,.08,.08)
    camera=bpy.data.objects.new('review-camera',bpy.data.cameras.new('review-camera')); scene.collection.objects.link(camera)
    scene.camera=camera; camera.data.type='ORTHO'; camera.data.ortho_scale=9
    for tag,position in [('front',(0,-10,1)),('side',(8,-8,4)),('above',(0,-9,8)),('low',(0,-10,-1))]:
        camera.location=position; camera.rotation_euler=(Vector((0,0,0))-camera.location).to_track_quat('-Z','Y').to_euler()
        scene.render.filepath=str(ROOT/f'review-{tag}.png'); bpy.ops.render.render(write_still=True)
    for obj in objects: obj.show_wire=True; obj.show_all_edges=True
    scene.render.filepath=str(ROOT/'review-wire.png'); bpy.ops.render.render(write_still=True)
    for obj in objects: obj.show_wire=False

def main():
    print('BLENDER_VERSION',bpy.app.version_string)
    ROOT.mkdir(parents=True,exist_ok=True)
    for obj in list(bpy.data.objects): bpy.data.objects.remove(obj,do_unlink=True)
    mats=material_setup(); objects=[]; stats=[]
    for name,mesh,x in [('rock',rock(),-3),('chest',chest(),0),('tree',tree(),3)]:
        obj,s=export(name,mesh,mats,x); objects.append(obj); stats.append(s)
    bpy.context.view_layer.update(); bpy.context.preferences.filepaths.save_version=0
    review(objects)
    bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'assets.blend'))
    (ROOT/'materials.json').write_text(json.dumps(MATERIALS,indent=2))
    print('RESULT '+json.dumps(stats))

if __name__=='__main__': main()
