"""Same-input receipts for terminal test work; never a substitute for products."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


def command(*args):
    return subprocess.check_output(args, stderr=subprocess.DEVNULL, timeout=10).decode().strip()


def file_hash(path):
    with path.open('rb') as stream:
        digest = hashlib.sha256()
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def fingerprint(scope):
    # The initial implementation admits only the single-device Linux L4 tier.
    # Other platforms, unmanaged tools and unknown inputs run normally.
    if (sys.platform != 'linux' or os.environ.get('OG_RENDERER') == 'none'
            or any(os.environ.get(k) for k in ('OG_GFX_CAPTURE', 'OG_BIN', 'OG_ENGINE_PACKAGE'))):
        return None
    logs = Path(os.environ['GATE_LOGS'])
    run = json.loads((logs / 'run.json').read_text())
    if not run['force'] or run['skipped'] or not run['tree']:
        return None
    if command('git', 'rev-parse', 'HEAD^{tree}') != run['tree'] or command('git', 'status', '--porcelain'):
        return None
    gpu = command('nvidia-smi', '--query-gpu=name,uuid,driver_version', '--format=csv,noheader').splitlines()
    if len(gpu) != 1 or not gpu[0].startswith('NVIDIA L4,'):
        return None
    runtime = {}
    for tool, version in [('bun', '--revision'), ('rustc', '-vV'), ('cargo', '--version'),
                          ('tsrs', '--version'), ('wasm-bindgen', '--version')]:
        path = shutil.which(tool)
        if not path:
            return None
        runtime[tool] = (str(Path(path).resolve()), file_hash(Path(path)), command(tool, version))
    if scope == 'browser':
        # Resolve Chrome using the runner's own lookup, including a newer
        # agent-browser install replacing the previous default.
        chrome = Path(command('bun', '-e', 'import {chromePath} from "./packages/engine/tests/web/cdp.ts"; console.log(chromePath())')).resolve(strict=True)
        runtime['chrome'] = (str(chrome), file_hash(chrome), command(str(chrome), '--version'))
        for tool in ('wasm-split', 'rustfilt'):
            splitter = shutil.which(tool)
            if not splitter:
                return None
            runtime[tool] = (file_hash(Path(splitter)), command(splitter, '--version'))
    engine = Path('packages/engine')
    products = {}
    roots = [engine / 'native', *sorted((engine / 'dist').glob('web*'))]
    if len(roots) < 2 or any(not root.is_dir() or root.is_symlink() for root in roots):
        return None
    for root in roots:
        for path in sorted(root.rglob('*')):
            if path.is_symlink():
                return None
            if path.is_file():
                products[str(path)] = file_hash(path)
    if not any(name.endswith('.node') for name in products) or not any(name.endswith('.wasm') for name in products):
        return None
    # Do not persist environment values. Drop only gate bookkeeping that
    # naturally differs between attempts; a user setting changing is a miss.
    volatile = {'GATE_LOGS', 'OG_GATE_RESUME', 'OG_GATE_RAN', 'OG_GPU_SLOT', 'OG_LOCK_IDENTITIES', '_', 'SHLVL'}
    scratch = Path.cwd() / '.og' / 'tmp'
    for name in ('TMPDIR', 'TMP', 'TEMP'):
        value = Path(os.environ.get(name, ''))
        if value.parent == scratch and value.name.startswith('run.'):
            volatile.add(name)
    env = {k: v for k, v in os.environ.items() if k not in volatile}
    inputs = [1, scope, run, str(Path.cwd()), env, runtime, gpu, products,
              Path('/proc/sys/kernel/random/boot_id').read_text().strip()]
    return hashlib.sha256(json.dumps(inputs, sort_keys=True).encode()).hexdigest()


def main():
    action, scope, *args = sys.argv[1:]
    if scope not in ('browser', 'bun-test'):
        return 1
    logs = Path(os.environ['GATE_LOGS'])
    if action == 'key':
        key = fingerprint(scope)
        if key:
            print(key)
        return
    name, key = args
    if not re.fullmatch(r'[a-z0-9-]+', name) or not re.fullmatch(r'[a-f0-9]{64}', key):
        return 1
    relative = f'{scope}-{name}.receipt'
    if action == 'reuse':
        source = os.environ.get('OG_GATE_RESUME')
        if not source or not re.fullmatch(r'[0-9]+-[0-9]+', source) or (logs.parent / source / relative).read_text().strip() != key:
            return 1
        print(f'REUSED {scope}:{name} from {source} (exact input receipt)', flush=True)
    elif action != 'pass':
        return 1
    receipt = logs / relative
    pending = receipt.with_suffix('.part')
    pending.write_text(key + '\n')
    pending.replace(receipt)


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, subprocess.SubprocessError, ValueError, KeyError):
        sys.exit(1)
