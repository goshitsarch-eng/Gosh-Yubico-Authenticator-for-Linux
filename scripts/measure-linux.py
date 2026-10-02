#!/usr/bin/env python3
"""Measure observed no-device readiness and process-tree memory on a real X11 desktop.

Run under xvfb-run/dbus-run-session with pcscd available. This is a warm-cache
development measurement, not a hardware/scrolling or native-platform benchmark.
"""
import argparse
import json
import os
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path


def memory_tree(parent):
    parents = {}
    for path in Path('/proc').glob('[0-9]*/status'):
        try:
            fields=dict(line.split(':', 1) for line in path.read_text().splitlines() if ':' in line)
            parents[int(path.parent.name)]=int(fields['PPid'].strip())
        except (OSError, ValueError, KeyError):
            continue
    selected={parent}
    while True:
        added={pid for pid, ppid in parents.items() if ppid in selected} - selected
        if not added:
            break
        selected.update(added)
    pss=0
    for pid in selected:
        for line in Path(f'/proc/{pid}/smaps_rollup').read_text().splitlines():
            if line.startswith('Pss:'):
                pss+=int(line.split()[1])
    return {'processes':len(selected), 'pss_mib':round(pss/1024, 1)}


def measure(binary, ready_text):
    with tempfile.TemporaryDirectory(prefix='gosh-performance-') as directory:
        config=Path(directory)/'gosh-authenticator'
        config.mkdir()
        (config/'settings.json').write_text('{"theme_mode":"light"}')
        environment=dict(os.environ, XDG_CONFIG_HOME=directory)
        with (Path(directory)/'app.log').open('w+') as log:
            started=time.monotonic()
            process=subprocess.Popen([str(binary)], env=environment, stdout=log, stderr=log)
            try:
                while time.monotonic()-started<20:
                    result=subprocess.run([sys.executable, str(Path(__file__).with_name('desktop-probe.py'))],
                                          input=json.dumps({'name':ready_text}), text=True,
                                          capture_output=True, timeout=5)
                    if result.returncode==0:
                        ready=time.monotonic()-started
                        break
                    if result.returncode!=3 or process.poll() is not None:
                        log.seek(0)
                        raise RuntimeError(f'Readiness failed: {result.stderr}\n{log.read()}')
                else:
                    raise TimeoutError('No-device readiness did not appear')
                time.sleep(2)
                return {'ready_seconds':round(ready, 3), **memory_tree(process.pid)}
            finally:
                process.terminate()
                process.wait(timeout=5)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('--ready-text', default='No YubiKey connected')
    parser.add_argument('--samples', type=int, default=3)
    arguments=parser.parse_args()
    values=[measure(arguments.binary.resolve(strict=True), arguments.ready_text) for _ in range(arguments.samples)]
    print(json.dumps({'samples':values, 'median_ready_seconds':statistics.median(v['ready_seconds'] for v in values),
                      'median_pss_mib':statistics.median(v['pss_mib'] for v in values)}, indent=2))
