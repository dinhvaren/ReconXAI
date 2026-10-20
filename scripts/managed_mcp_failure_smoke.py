"""Actual running-install cancellation, failed-update preservation and runtime selection.
Reuses only a previous isolated smoke workspace; never normal user data.
"""
import argparse,subprocess,os,json,time,sqlite3,re,shutil
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--data',required=True);p.add_argument('--existing-runtime',required=True);p.add_argument('--approve-install',action='store_true');a=p.parse_args();assert a.approve_install
d=Path(a.data).resolve();assert d.name.startswith('mcp-cli-');d.mkdir(parents=True,exist_ok=False)
user_runtime=d/'user-runtime';user_runtime.mkdir();node=user_runtime/('node.exe' if os.name=='nt' else 'node');shutil.copy2(Path(a.existing_runtime).resolve(),node);node.chmod(0o700)
binary=str(Path(a.binary).resolve());env=dict(os.environ,RECONX_DATA_DIR=str(d),PATH=str(user_runtime)+os.pathsep+os.environ.get('PATH',''));calls=[];checks=[];ID='reconx.mcp-everything'
def run(args,expected=None):
    proc=subprocess.run([binary,'--json',*args],env=env,capture_output=True,text=True,encoding='utf-8',timeout=650);calls.append({'args':args,'exit':proc.returncode,'stdout':proc.stdout,'stderr':proc.stderr})
    (d/'negative-calls.json').write_text(json.dumps(calls,indent=2),encoding='utf-8')
    if expected:assert proc.returncode and expected in proc.stdout+proc.stderr,calls[-1];return
    assert not proc.returncode,calls[-1];return json.loads(proc.stdout) if proc.stdout.strip() else None
def plan(version,managed=False):return run(['extensions','plan',ID,'--version',version,*(['--managed-runtime'] if managed else [])])
def install(version,managed=False,wait=True):
    value=plan(version,managed);return run(['extensions','install',ID,'--version',version,'--fingerprint',value['fingerprint'],'--approve',*(['--managed-runtime'] if managed else []),*(['--wait'] if wait else [])])
worker=None;result={'status':'failed','checks':checks}
try:
    assert run(['extensions','info',ID])['status']=='not_installed'
    run(['project','create','Owned installation failure acceptance'])
    entry=install('2.0.0');install('1.0.0');entry=install('2.0.0');server=entry['managed']['server_id'];previous=run(['mcp','list'])[0]['revision']
    job=install('1.0.0',managed=True,wait=False)
    worker=subprocess.Popen([binary,'--json','jobs','run','--watch'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    observed=None
    for i in range(250):
        current=run(['jobs','show',job['id']])
        if current['status']=='running' and any(re.search(r'Downloading verified dependency: [1-9][0-9]*[/ ]',line) for line in current['logs']):observed=current;break
        if current['status'] in ['failed','completed','cancelled']:raise AssertionError(current)
        time.sleep(.03)
    assert observed,'Did not observe actual running dependency I/O'
    run(['jobs','cancel',job['id']])
    for i in range(200):
        current=run(['jobs','show',job['id']])
        if current['status']=='cancelled':break
        time.sleep(.05)
    assert current['status']=='cancelled',current
    worker.terminate();worker.wait(timeout=15);worker=None
    retained=run(['extensions','info',ID]);assert retained['status']=='ready' and retained['installed']['version']=='2.0.0',retained
    assert run(['mcp','list'])[0]['revision']==previous
    checks.append('Actual running dependency I/O cancelled; previous Ready package/config revision preserved after process reopen')
    # Cached v1 exists from the earlier positive lifecycle. Switching to a
    # managed runtime must bind the reused tree to the newly approved executable.
    approved=plan('1.0.0',True);entry=install('1.0.0',True);actual=run(['mcp','list'])[0]
    assert actual['config']['executable']==approved['package_plan']['runtime']['executable'],actual
    conn=sqlite3.connect(d/'reconx.db');desc=json.loads(conn.execute('SELECT descriptor_json FROM managed_mcp_extensions').fetchone()[0]);conn.close()
    assert desc['receipt']['runtime']['managed'] and desc['receipt']['runtime']['version']=='24.22.0'
    checks.append('Cached package reuse switches to the exact newly approved managed Node path/hash; actual MCP health passes')
    # Restore v2 as active, then corrupt only the inactive cached v1 tree.
    install('2.0.0',True);before=run(['mcp','list'])[0]['revision'];old=Path(desc['receipt']['directory']);changed=old/next(iter(desc['receipt']['files']));original=changed.read_bytes();changed.write_bytes(original+b'\nnegative integrity fixture')
    value=plan('1.0.0',True)
    run(['extensions','install',ID,'--version','1.0.0','--managed-runtime','--fingerprint',value['fingerprint'],'--approve','--wait'],expected='PACKAGE_INTEGRITY')
    entry=run(['extensions','info',ID]);assert entry['status']=='ready' and entry['installed']['version']=='2.0.0',entry
    assert run(['mcp','list'])[0]['revision']==before;changed.write_bytes(original)
    checks.append('Real failed update rejects corrupted inactive package and atomically retains previous Ready config/version')
    run(['extensions','remove',ID]);result={'status':'passed','checks':checks,'commands':len(calls)}
except Exception as e:result={'status':'failed','checks':checks,'error':str(e),'commands':len(calls)}
finally:
    if worker is not None:worker.terminate();worker.wait(timeout=15)
    (d/'negative-results.json').write_text(json.dumps(result,indent=2),encoding='utf-8');print(json.dumps(result,indent=2))
raise SystemExit(result['status']!='passed')
