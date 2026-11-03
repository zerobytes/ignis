"""Cottage-specific selection policy; all transformations execute in Ignis."""


def lantern_edits(call, wall_z, offset):
    """Select complete connected pieces by inspected bounds, never client vertices."""
    edits=[];selection={}
    limits=[[2.05,2.20,wall_z+.28],[2.70,3.32,wall_z+.57]]
    for asset,expected in [('bronze',9),('flowers',1)]:
        inspected=call('inspect_mesh_islands',asset=asset)
        selected=[s for s in inspected['topology']['islands'] if all(limits[0][axis]<=s['bounds'][0][axis] and s['bounds'][1][axis]<=limits[1][axis] for axis in range(3))]
        if len(selected)!=expected:raise ValueError(f'Lantern selection changed in {asset}: expected {expected} islands, got {len(selected)}; inspect before editing')
        selection[asset]=selected
        edits.append({'op':'mesh','edit':{'kind':'transform_islands','asset':asset,'reference':inspected['reference'],'islands':[s['id'] for s in selected],'transform':{'pivot':[2.4,2.57,wall_z+.42],'translation':offset,'rotation':[0,0,0,1],'scale':1}}})
    return edits,selection


def adjust_lantern(call,wall_z,offset):
    if offset==[0,0,0]:return {}
    edits,selection=lantern_edits(call,wall_z,offset)
    before=call('inspect');predicted=call('preview_scene_edits',edits=edits)['scene'];assert call('inspect')==before
    call('edit_scene',edits=edits);assert call('inspect')['scene']==predicted
    return selection
