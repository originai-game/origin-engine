"""Optional, bounded cache of bindgen or split output trees under the gate lock."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

from continuation import file_hash


def manifest(path):
    if path.is_file() and not path.is_symlink():
        return {'.': file_hash(path)}
    if path.is_symlink() or not path.is_dir():
        raise ValueError('missing or linked product')
    entries = list(path.rglob('*'))
    if any(p.is_symlink() or not (p.is_file() or p.is_dir()) for p in entries):
        raise ValueError('unexpected product type')
    files = {str(p.relative_to(path)): file_hash(p) for p in entries if p.is_file()}
    if not files:
        raise ValueError('empty product')
    return files


def key(kind, variant, source, command):
    tools = ('wasm-bindgen',) if kind == 'bindgen' else ('bun', 'wasm-split')
    tool_hashes = [file_hash(Path(shutil.which(tool) or '')) for tool in tools]
    # Splitting executes engine TS via the profile program. Use the complete
    # source tree for this experiment until its closure is proven.
    tree = os.environ.get('OG_GATE_TREE') if kind == 'split' else None
    if kind == 'split' and not tree:
        raise ValueError('no source snapshot for splitter')
    env = {k: v for k, v in os.environ.items() if k.startswith(('BUN_', 'WASM_', 'WBG_'))}
    inputs = [1, kind, variant, str(source), command, tool_hashes, manifest(source), tree, env]
    return hashlib.sha256(json.dumps(inputs, sort_keys=True).encode()).hexdigest()


def main():
    kind, variant, source, output, *command = sys.argv[1:]
    assert kind in ('bindgen', 'split') and re.fullmatch(r'[a-zA-Z0-9_-]+', variant)
    source, output = Path(source), Path(output)
    cache = Path(os.environ['OG_TARGET_DIR']) / 'og-wasm-products' / kind / variant
    product = cache / 'product'
    try:
        wanted = key(kind, variant, source, command)
    except (OSError, ValueError):
        wanted = None
    if wanted:
        try:
            saved = json.loads((cache / 'receipt.json').read_text())
            if saved == {'key': wanted, 'files': manifest(product)}:
                shutil.copytree(product, output, dirs_exist_ok=True)
                print(f'{kind} {variant}: cache hit', flush=True)
                return 0
        except (OSError, ValueError):
            # A failed restore may have left a partial directory.
            shutil.rmtree(output, ignore_errors=True)
    print(f'{kind} {variant}: cache miss', flush=True)
    rc = subprocess.call(command)
    if rc or not wanted:
        return rc
    try:
        cache.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=cache.parent) as scratch:
            pending = Path(scratch) / variant
            shutil.copytree(output, pending / 'product')
            (pending / 'receipt.json').write_text(json.dumps({'key': wanted, 'files': manifest(pending / 'product')}))
            # All gate readers of this checkout hold the same target lock.
            if cache.exists():
                shutil.rmtree(cache)
            pending.rename(cache)
    except (OSError, ValueError) as exc:
        print(f'{kind} {variant}: could not cache output ({exc})', file=sys.stderr)
    return 0


if __name__ == '__main__':
    sys.exit(main())
