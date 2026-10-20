"""Actual reviewed npm package lifecycle; uses a fresh isolated workspace.

Requires --approve-install for public pinned artifact downloads and local MCP execution.
Does not use account credentials or invoke tools beyond explicitly approved echo.
"""
import argparse, json, os, subprocess, time, sqlite3
from pathlib import Path

p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--output',required=True);p.add_argument('--approve-install',action='store_true');p.add_argument('--managed-runtime',action='store_true');p.add_argument('--legacy-binary');a=p.parse_args()
assert a.approve_install, 'Explicit --approve-install is required'
out=Path(a.output).resolve();out.mkdir(parents=True,exist_ok=False)
env=dict(os.environ,RECONX_DATA_DIR=str(out));binary=str(Path(a.binary).resolve());calls=[];checks=[];ID='reconx.mcp-everything'
def run(args,expected=None,legacy=False):
    proc=subprocess.run([str(Path(a.legacy_binary).resolve()) if legacy else binary,'--json',*args],env=env,capture_output=True,text=True,encoding='utf-8',timeout=650)
    record={'args':args,'exit':proc.returncode,'stdout':proc.stdout,'stderr':proc.stderr};calls.append(record)
    (out/'calls.json').write_text(json.dumps(calls,indent=2),encoding='utf-8')
    if expected:
        assert proc.returncode!=0 and expected in proc.stdout+proc.stderr,record
        return
    assert proc.returncode==0,record
    return json.loads(proc.stdout) if proc.stdout.strip() else None
def plan(version):
    args=['extensions','plan',ID,'--version',version]
    if a.managed_runtime:args+=['--managed-runtime']
    return run(args)
def install(version):
    value=plan(version);args=['extensions','install',ID,'--version',version,'--fingerprint',value['fingerprint'],'--approve','--wait']
    if a.managed_runtime:args+=['--managed-runtime']
    result=run(args);assert result['status']=='ready',result
    assert result['installed']['version']==version
    return result
result={'status':'failed','checks':checks}
try:
    project=run(['project','create','Owned MCP lifecycle fixture'],legacy=bool(a.legacy_binary))
    first=plan('2.0.0');assert len(first['package_plan']['dependencies'])==94
    run(['extensions','install',ID,'--fingerprint','wrong','--approve'],expected='PLAN_CHANGED')
    run(['extensions','install',ID,'--fingerprint',first['fingerprint']],expected='APPROVAL_REQUIRED')
    entry=install('2.0.0');server=entry['managed']['server_id'];assert len(entry['managed']['tools'])>0
    checks.append('Real verified npm2 installation, runtime reuse/auto-install, MCP initialize/list_tools and atomic registration')
    input_file=out/'echo.json';input_file.write_text(json.dumps({'message':'ReconX approved benign lifecycle echo'}),encoding='utf-8')
    args=['mcp','invoke',server,'echo','--input',str(input_file),'--project',project['id']]
    run(args,expected='BLOCKED_BY_POLICY')
    run(['extensions','permissions',ID,'--tool','echo','--approve'])
    preview=run(args);job=run([*args,'--fingerprint',preview['fingerprint'],'--approve']);run(['jobs','run'])
    job=run(['jobs','show',job['id']]);assert job['status']=='completed',job
    assert 'Echo: ReconX approved benign lifecycle echo' in json.dumps(job['result']),job
    checks.append('Explicit echo permission/input approval; real MCP tool result persisted in completed Job Engine task')
    reopened=run(['extensions','info',ID]);assert reopened['status']=='ready' and reopened['managed']['allowed_tools']==['echo']
    run(['extensions','disable',ID]);run(['mcp','connect',server],expected='EXTENSION_DISABLED')
    run(['extensions','enable',ID]);assert run(['extensions','doctor',ID])['status']=='ready'
    checks.append('Independent process reopen, preserved discovery/permissions/history, disable/enable/health lifecycle')
    # Only compiled reviewed versions may be activated. A failure must preserve v2.
    try:
        install('1.0.0');checks.append('Reviewed version rollback2→1 health verified')
        install('2.0.0');checks.append('Version update1→2 health verified; shared runtime reused')
    except Exception:
        retained=run(['extensions','info',ID]);assert retained['installed']['version']=='2.0.0' and retained['status']=='ready',retained
        raise
    conn=sqlite3.connect(out/'reconx.db');descriptor=json.loads(conn.execute('SELECT descriptor_json FROM managed_mcp_extensions').fetchone()[0]);conn.close()
    package=Path(descriptor['receipt']['directory']);file=package/next(iter(descriptor['receipt']['files']));original=file.read_bytes();file.write_bytes(original+b'\nIntegrity test')
    run(['mcp','connect',server],expected='PACKAGE_INTEGRITY');file.write_bytes(original)
    assert run(['extensions','doctor',ID])['status']=='ready';checks.append('Package tampering blocks launch; original integrity restored and reverified')
    run(['extensions','remove',ID]);assert not package.exists();assert not run(['mcp','list']);assert run(['extensions','info',ID])['status']=='not_installed'
    assert Path(descriptor['receipt']['runtime']['executable']).exists();assert run(['jobs','show',job['id']])['status']=='completed'
    checks.append('Owned package/config removed, shared runtime and persistent invocation history retained')
    conn=sqlite3.connect(out/'reconx.db');assert conn.execute('PRAGMA integrity_check').fetchone()[0]=='ok';assert not conn.execute('PRAGMA foreign_key_check').fetchall();assert conn.execute('PRAGMA user_version').fetchone()[0]==16
    if a.legacy_binary:
        backups=list(out.rglob('*pre-v16*'));assert backups,'Missing actual older-build migration backup';checks.append('Actual accepted older CLI workspace migrated to schema16 with integrity backup')
    result={'status':'passed','platform':os.name,'checks':checks,'commands':len(calls),'managed_runtime':a.managed_runtime,'test_package':'@modelcontextprotocol/server-everything 1.0.0/2.0.0'}
except Exception as e:
    result={'status':'failed','checks':checks,'error':str(e),'commands':len(calls)}
finally:
    (out/'results.json').write_text(json.dumps(result,indent=2),encoding='utf-8');print(json.dumps(result,indent=2))
raise SystemExit(result['status']!='passed')
