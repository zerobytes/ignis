"""Build-time collection choices shared by native botanical asset tools."""
from groundcover_assets import DEST, FAMILIES, ROOT, library


def collection(name):
    if name=='groundcover':
        return dict(destination=DEST,families=FAMILIES,generate=library,output=ROOT/'artifacts/groundcover',
                    recipe=ROOT/'examples/groundcover-field.recipe.json',title='Gramas e matos',columns=4,spacing=[2.2,2.8],variant_gap=.48,
                    gallery_bounds=[[-4.5,0,-3.7],[4.5,1.5,3.7]],gallery_direction=[.5,.95,1],
                    close_camera=dict(position=[1.8,1.1,5.0],target=[-.3,.35,0],fov_degrees=57),field_target=[0,.25,0],
                    foliage_weights=dict(grama_baixa=.45,grama_fina=.30,folha_larga=.15,capim_arqueado=.05,capim_seco=.05),
                    foliage_title='Somente gramas, sem pendões ou flores')
    if name=='tallgrass':
        from tallgrass_assets import DEST as tall_dest, FAMILIES as tall_families, library as tall_library
        return dict(destination=tall_dest,families=tall_families,generate=tall_library,output=ROOT/'artifacts/tallgrass',
                    recipe=ROOT/'examples/tallgrass-field.recipe.json',title='Mato alto',columns=2,spacing=[5.5,3.5],variant_gap=1.4,
                    gallery_bounds=[[-5.2,0,-6.5],[5.2,3.5,6.5]],gallery_direction=[.45,.70,1],
                    close_camera=dict(position=[2.6,1.7,6.0],target=[-.2,1.5,0],fov_degrees=57),field_target=[0,1.1,0],
                    foliage_weights=dict(capim_muralha=.45,capim_longo=.25,canico=.18,mato_ramos=.12),
                    foliage_title='Somente folhagens altas, sem flores ou pendões')
    raise ValueError(f'Unknown botanical collection: {name}')
