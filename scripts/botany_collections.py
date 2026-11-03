"""Build-time collection choices shared by native botanical asset tools."""
from groundcover_assets import DEST, FAMILIES, ROOT, library
COLLECTIONS=('groundcover','tallgrass','shrubs','colored-shrubs','reference-shrubs')


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
    if name=='shrubs':
        from shrub_assets import DEST as shrub_dest, FAMILIES as shrub_families, library as shrub_library
        return dict(destination=shrub_dest,families=shrub_families,generate=shrub_library,output=ROOT/'artifacts/shrubs',
                    recipe=ROOT/'examples/shrubs-field.recipe.json',title='Arbustos',columns=2,spacing=[6.2,3.8],variant_gap=1.65,
                    gallery_bounds=[[-6.2,0,-7.0],[6.2,2.3,7.0]],gallery_direction=[.45,.75,1],
                    close_camera=dict(position=[3.8,2.1,6.5],target=[0,.65,0],fov_degrees=54),field_target=[0,.65,0],
                    foliage_weights=dict(compacto=.30,aberto=.25,espalhado=.20,vertical=.15,folha_grande=.10),
                    foliage_title='Arbustos sem flores',density_compare=['compacto','aberto'])
    if name=='colored-shrubs':
        from colored_shrub_assets import DEST as colored_dest, FAMILIES as colored_families, library as colored_library
        return dict(destination=colored_dest,families=colored_families,generate=colored_library,output=ROOT/'artifacts/colored-shrubs',
                    recipe=ROOT/'examples/colored-shrubs-field.recipe.json',title='Arbustos de folhagem colorida',columns=2,spacing=[5.4,4.0],variant_gap=1.4,
                    gallery_bounds=[[-5.1,0,-3.8],[5.1,2.2,3.8]],gallery_direction=[.40,.80,1],
                    close_camera=dict(position=[3.8,2.1,6.5],target=[0,.65,0],fov_degrees=54),field_target=[0,.70,0],
                    foliage_weights=dict(pingo_ouro=.25,berberis_vinho=.25,loropetalo_roxo=.25,cotinus_roxo=.25),
                    foliage_title='Somente folhagem colorida, sem flores')
    if name=='reference-shrubs':
        from reference_shrub_assets import DEST as ref_dest, FAMILIES as ref_families, library as ref_library
        return dict(destination=ref_dest,families=ref_families,generate=ref_library,output=ROOT/'artifacts/reference-shrubs',
                    recipe=ROOT/'examples/reference-shrubs-field.recipe.json',title='Arbustos de deserto e topiarias',columns=2,spacing=[5.6,3.2],variant_gap=1.5,compact_samples=True,
                    gallery_bounds=[[-5.6,0,-5.7],[5.6,1.5,5.7]],gallery_direction=[.45,.9,1],
                    close_camera=dict(position=[4.0,2.3,6.0],target=[0,.55,0],fov_degrees=54),field_target=[0,.45,0],
                    foliage_weights={f['id']:1. for f in ref_families},foliage_title='Oito perfis em plantio misto')
    raise ValueError(f'Unknown botanical collection: {name}')
