"""Compile a binary asset, exercise an isolated decoder, then evaluate it against the source."""
import hashlib,json,os,shutil,subprocess,sys,tempfile
from pathlib import Path
root=Path('artifacts/appearance-probe/compiled-rock');root.mkdir(parents=True,exist_ok=True)
binary=Path(os.environ.get('CARGO_TARGET_DIR','target'))/'release/examples/appearance_probe'
binary=binary.resolve()
subprocess.run(['cargo','build','--locked','--release','--example','appearance_probe'],check=True)
paths=[Path('artifacts/appearance-probe/assets/rock')/f for f in ['vertices.bin','indices.bin']]
paths += [Path('examples/appearance_probe')/f for f in ['rock_symbol.wgsl','rock_symbol.rs','compiled_rock.rs','compiled_rock_data.rs','compiled_rock.wgsl']]
(root/'input-manifest.json').write_text(json.dumps({str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},indent=2)+'\n')
subprocess.run([binary,'--compile-rock'],check=True)
with tempfile.TemporaryDirectory(prefix='ignis-compiled-decoder-') as tmp:
    isolated=Path(tmp)/root;isolated.mkdir(parents=True)
    for i in range(2):(isolated/f'rock{i}.irk').symlink_to((root/f'rock{i}.irk').resolve())
    subprocess.run([binary,'--compiled-rock-load'],cwd=tmp,check=True)
    for name in ['load.json','standalone7.png','standalone8.png','standalone9.png','standalone10.png']:shutil.move(isolated/name,root/name)
subprocess.run([binary,'--compiled-rock'],check=True)
for script in ['check_compiled_rock.py','report_compiled_rock.py']:subprocess.run([sys.executable,f'scripts/{script}'],check=True)
subprocess.run(['node','scripts/check_compiled_rock_ui.mjs'],check=True)
