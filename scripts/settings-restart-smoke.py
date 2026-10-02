#!/usr/bin/env python3
"""Check actual preference controls before and after a fresh app process."""
import json, os, subprocess, sys, time
from pathlib import Path
repo=Path(__file__).resolve().parents[1]
settings_path=Path(os.environ.get('GOSH_RESTART_SETTINGS', str(Path(os.environ['GOSH_SMOKE_DIR'])/'settings.json')))
def probe(name,operation='query',**extra):
    result=subprocess.run([sys.executable,str(repo/'scripts/desktop-probe.py')],input=json.dumps(dict(name=name,operation=operation,**extra)),text=True,capture_output=True,timeout=10)
    assert result.returncode==0,(name,result.stdout,result.stderr)
    return json.loads(result.stdout)
deadline=time.monotonic()+20
while time.monotonic()<deadline:
    result=subprocess.run([sys.executable,str(repo/'scripts/desktop-probe.py')],input=json.dumps({'name':'No YubiKey connected'}),text=True,capture_output=True,timeout=10)
    if result.returncode==0:break
else:raise AssertionError('No-device page not ready')
probe('Settings','activate');time.sleep(.5)
if os.environ.get('GOSH_PERSIST_PHASE')=='save':
    assert probe('Theme','selected')['selected']==['Light']
    assert probe('Clear clipboard','selected')['selected']==['60 seconds']
    probe('Theme','select',index=2); time.sleep(.5)
    probe('Clear clipboard','select',index=4);time.sleep(.5)
    probe('Prompt to unlock on launch','activate');time.sleep(.5)
    probe('Allow optional favicon downloads','activate');time.sleep(.5)
    settings=json.loads(settings_path.read_text())
    assert settings['theme_mode']=='dark'
    assert settings['clipboard_timeout_seconds']==120
    assert settings['require_pin_on_launch'] and settings['allow_favicons']
    assert settings['future_setting']=={'v':1}
assert probe('Theme','selected')['selected']==['Dark']
assert probe('Clear clipboard','selected')['selected']==['120 seconds']
assert probe('Prompt to unlock on launch')['checked']
assert probe('Allow optional favicon downloads')['checked']
print('Restarted application restored Dark, 120 seconds, protected-key prompt and favicon preference from persisted settings')
if os.environ.get('GOSH_SMOKE_QUIT'):
    subprocess.run(['xdotool','key','ctrl+q'],check=True)
