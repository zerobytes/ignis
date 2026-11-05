#!/usr/bin/env python3
"""Build, verify or open the native Land of Survivors character front end."""
import argparse,json,os,subprocess,sys
from pathlib import Path
PROJECT=Path(__file__).resolve().parent
ENGINE=PROJECT.parents[1]
def main():
 p=argparse.ArgumentParser(description=__doc__)
 p.add_argument('--build',action='store_true')
 p.add_argument('--verify',action='store_true')
 p.add_argument('--open',action='store_true')
 a=p.parse_args();binary=ENGINE/'target/release/examples/land_of_survivors_app'
 out=PROJECT/'artifacts/app';out.mkdir(parents=True,exist_ok=True)
 if a.build:
  source=ENGINE/'artifacts/character-base/human-nose.ignis-character'
  if not source.is_file():
   for script in ['prepare_source','extend_source','refine_source','smooth_source','cupid_source','nose_source']:
    subprocess.run([sys.executable,str(ENGINE/f'scripts/character/{script}.py')],cwd=ENGINE,check=True)
  subprocess.run(['cargo','build','--locked','--release','--example','land_of_survivors_app'],cwd=ENGINE,check=True)
 if not binary.is_file():p.error('Build first: python3 games/land-of-survivors/app.py --build')
 if a.verify:subprocess.run([str(binary),'--verify'],cwd=ENGINE,check=True);return
 if a.build and not a.open:return
 with (out/'window.log').open('w') as log:
  process=subprocess.Popen([str(binary)],cwd=ENGINE,stdin=subprocess.DEVNULL,stdout=log,stderr=log,start_new_session=True)
 (out/'launch.json').write_text(json.dumps({'pid':process.pid,'binary':str(binary),'profile_override':os.environ.get('LOS_PROFILE_PATH'),'entry_scene':'scenes/Inicio.scene.json'},indent=2)+'\n')
 print(f'Land of Survivors opened (PID {process.pid}).')
if __name__=='__main__':main()
