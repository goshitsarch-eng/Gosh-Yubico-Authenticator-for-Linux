#!/usr/bin/env python3
"""Exercise the real Linux desktop UI via AT-SPI; no simulated device is used."""
import json
import os
import subprocess
import sys
import time
from pathlib import Path

PROBE = Path(__file__).with_name('desktop-probe.py')

def probe(name, operation='query', **extra):
    return subprocess.run([sys.executable, str(PROBE)], input=json.dumps(dict(name=name, operation=operation, **extra)), text=True, capture_output=True, timeout=10)

def find(name, timeout=20, **extra):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        result=probe(name, **extra)
        if result.returncode==0:
            return json.loads(result.stdout)
        if result.returncode!=3:
            raise AssertionError(result.stderr)
    raise AssertionError(f'Missing {name!r}')

def activate(name):
    find(name)
    result=probe(name, 'activate')
    assert result.returncode==0, (name,result.stderr)
    time.sleep(.4)


def change(name, operation, **extra):
    result=probe(name, operation, **extra)
    assert result.returncode==0, (name, result.stderr)
    time.sleep(.5)


def screenshot(name):
    destination=os.environ.get('GOSH_SCREENSHOTS')
    if not destination:
        return
    from PIL import ImageGrab
    window=subprocess.check_output(['xdotool','search','--onlyvisible','--name','^Gosh Yubico Authenticator$'],text=True).splitlines()[-1]
    geometry=subprocess.check_output(['xdotool','getwindowgeometry','--shell',window],text=True)
    values=dict(line.split('=',1) for line in geometry.splitlines() if '=' in line)
    x,y,w,h=(int(values[key]) for key in ['X','Y','WIDTH','HEIGHT'])
    directory=Path(destination)
    directory.mkdir(parents=True,exist_ok=True)
    ImageGrab.grab(bbox=(x,y,x+w,y+h)).save(directory/name)

find('No YubiKey connected')
screenshot('linux-credentials-light.png')
activate('Retry connection')
find('No YubiKey connected')
activate('Settings')
find('Appearance')
find('Clear clipboard')
find('Prompt to unlock on launch')
find('Allow optional favicon downloads')
assert find('Clear clipboard',operation='selected')['selected']==['30 seconds']
assert not find('Change password…')['enabled']
change('Theme', 'select', index=2)
screenshot('linux-settings-dark.png')
change('Theme', 'select', index=0)
change('Theme', 'select', index=1)
for index in range(5):
    change('Clear clipboard', 'select', index=index)
    assert find('Clear clipboard',operation='selected')['selected']==[f'{(10,20,30,60,120)[index]} seconds']
    if os.environ.get('GOSH_SMOKE_DIR'):
        assert json.loads((Path(os.environ['GOSH_SMOKE_DIR'])/'settings.json').read_text())['clipboard_timeout_seconds']==(10,20,30,60,120)[index]
change('Clear clipboard', 'select', index=3)
activate('Prompt to unlock on launch')
assert find('Prompt to unlock on launch')['checked']
activate('Prompt to unlock on launch')
activate('Allow optional favicon downloads')
assert find('Allow optional favicon downloads')['checked']
activate('Allow optional favicon downloads')
settings_dir=os.environ.get('GOSH_SMOKE_DIR')
if settings_dir:
    persisted=json.loads((Path(settings_dir)/'settings.json').read_text())
    assert persisted['theme_mode']=='light'
    assert persisted['clipboard_timeout_seconds']==60
    assert not persisted['require_pin_on_launch']
    assert not persisted['allow_favicons']
    assert persisted['future_setting']=={'v':1}
activate('About')
find('About Gosh')
screenshot('linux-about.png')
activate('License details')
find('Licenses')
find('Open third-party notices')
activate('Close dialog')
activate('Key Info')
find('Insert your YubiKey to view device information.')
subprocess.run(['xdotool','key','ctrl+f'],check=True)
time.sleep(.5)
subprocess.run(['xclip','-selection','clipboard'],input='服务',text=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,check=True)
subprocess.run(['xdotool','key','ctrl+v'],check=True)
time.sleep(.4)
assert find('Search credentials',role='entry')['value']=='服务'
assert probe('Could not focus search:',prefix=True).returncode==3
subprocess.run(['xdotool','key','ctrl+a','BackSpace'],check=True)
activate('Credentials')
activate('+ Add credential')
find('Account details')
find('Secret (Base32 or otpauth URI)')
find('TOTP period (seconds)')
change('Secret (Base32 or otpauth URI)', 'text', value='otpauth://totp/%E6%9C%8D%E5%8A%A1:%C3%A9?secret=JBSWY3DPEHPK3PXP&period=60&digits=8&algorithm=SHA256')
assert find('Issuer')['value']=='服务'
assert find('Account')['value']=='é'
assert find('TOTP period (seconds)')['value']=='60'
assert find('Digits',operation='selected')['selected']==['8']
assert find('Algorithm',operation='selected')['selected']==['SHA256']
change('Type', 'select', index=1)
find('Initial HOTP counter')
change('Initial HOTP counter', 'text', value='42')
change('Digits', 'select', index=2)
change('Algorithm', 'select', index=2)
activate('Require physical touch to generate a code')
assert find('Require physical touch to generate a code')['checked']
assert not find('Save credential')['enabled']
screenshot('linux-add-credential.png')
activate('Cancel')
find('Secure hardware. Simple access.')
subprocess.run(['xdotool', 'key', 'ctrl+n'], check=True)
find('Account details')
subprocess.run(['xdotool', 'key', 'Escape'], check=True)
find('Secure hardware. Simple access.')
window=subprocess.check_output(['xdotool','search','--onlyvisible','--name','^Gosh Yubico Authenticator$'],text=True).splitlines()[-1]
subprocess.run(['xdotool','windowsize',window,'420','600'],check=True)
time.sleep(.6)
find('Settings')
screenshot('linux-small-window.png')
subprocess.run(['xdotool','windowsize',window,'960','720'],check=True)
qr_fixture=os.environ.get('GOSH_QR_FIXTURE')
if qr_fixture:
    subprocess.run(['xdotool','windowfocus',window,'key','ctrl+i'],check=True)
    time.sleep(2)
    subprocess.run(['xdotool','key','ctrl+l'],check=True)
    subprocess.run(['xclip','-selection','clipboard'],input=qr_fixture,text=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,check=True)
    subprocess.run(['xdotool','key','ctrl+v','Return'],check=True)
    time.sleep(1)
    activate('Select')
    find('Account details')
    deadline=time.monotonic()+10
    while find('Issuer')['value']!='QR Regression' and time.monotonic()<deadline:
        time.sleep(.2)
    assert find('Issuer')['value']=='QR Regression'
    assert find('Account')['value']=='é'
    assert find('TOTP period (seconds)')['value']=='60'
    activate('Cancel')
    print('Native portal QR file selection and background import passed')
if os.environ.get('GOSH_SMOKE_QUIT'):
    subprocess.run(['xdotool','windowfocus',window,'key','ctrl+q'],check=True)
print('Dioxus desktop smoke passed: no-device, retry, navigation, preferences, About, credential form, keyboard shortcut and Escape')
