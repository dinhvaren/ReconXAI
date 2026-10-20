"""Streaming ingestion acceptance using generated benign padded PE/ELF fixtures.
Never executes an imported sample or contacts a network service.
"""
import argparse, datetime as dt, hashlib, json, os, platform, shutil, subprocess, signal
from pathlib import Path

def digest(path):
    h=hashlib.sha256()
    with path.open('rb') as source:
        while chunk:=source.read(1024*1024): h.update(chunk)
    return h.hexdigest()

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--sample',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--include-gib',action='store_true');p.add_argument('--resume',action='store_true');p.add_argument('--reanalyze',action='store_true');a=p.parse_args()
    binary=a.binary.resolve(strict=True); sample=a.sample.resolve(strict=True);out=a.output.resolve()
    if a.resume: assert out.name.startswith('large-import-cli-') and (out/'commands.json').is_file() and (out/'workspace/reconx.db').is_file()
    else: out.mkdir(parents=True,exist_ok=False)
    assert sample.stat().st_size<64*1024*1024
    sizes=[80,256,512]+([1024] if a.include_gib else [])
    assert shutil.disk_usage(out).free>sum(sizes)*1024*1024*2+2*1024**3,'Insufficient safe test space'
    env={**os.environ,'RECONX_DATA_DIR':str(out/'workspace')};calls=json.loads((out/'commands.json').read_text()) if a.resume else [];checks=[];imports=[]
    def command(*args,fail=False):
        r=subprocess.run([str(binary),'--json',*map(str,args)],env=env,text=True,encoding='utf-8',capture_output=True,timeout=240)
        calls.append({'arguments':list(map(str,args)),'exit_code':r.returncode,'timestamp':dt.datetime.now(dt.timezone.utc).isoformat()});(out/'commands.json').write_text(json.dumps(calls,indent=2))
        if fail: assert r.returncode;return
        if r.returncode:raise RuntimeError(r.stderr[:2000])
        return json.loads(r.stdout),r.stderr
    if a.resume:
        projects=command('project','list')[0];assert len(projects)==1 and projects[0]['name']=='Benign large-file acceptance';project=projects[0]
    else: project=command('project','create','Benign large-file acceptance')[0]
    for size in sizes:
        path=out/f'benign-{size}MiB{sample.suffix or ".elf"}'
        if not a.resume or not path.exists():
            shutil.copyfile(sample,path)
            with path.open('r+b') as f:f.truncate(size*1024*1024)
        assert path.stat().st_size==size*1024*1024
        before=digest(path)
        command('reverse','import',path,'--project',project['id'],'--max-mib','64','--authorized',fail=True)
        prior=next((f for f in command('reverse','list','--project',project['id'])[0] if f['name']==path.name and f['sha256']==before),None) if a.resume else None
        if prior: value=prior;progress=(out/f'progress-{size}.jsonl').read_text(encoding='utf-8')
        else:
            value,progress=command('reverse','import',path,'--project',project['id'],'--authorized','--progress')
            (out/f'progress-{size}.jsonl').write_text(progress,encoding='utf-8')
        observations=[json.loads(l) for l in progress.splitlines() if l.startswith('{')]
        assert observations and all(0<=r['completed']<=r['total'] for r in observations)
        assert value['size']==size*1024*1024 and value['sha256']==before
        blob=out/'workspace/imports'/f'{before}.bin'
        assert digest(blob)==before and digest(path)==before
        assert any(f['id']==value['id'] and f['size']==value['size'] for f in command('reverse','list','--project',project['id'])[0])
        imports.append(value);checks.append(f'{size} MiB complete streaming copy; independent hashlib checks of original/managed bytes; explicit budget rejection; persisted metadata')
    largest=imports[-1]
    analysis=command('reverse','metadata' if a.resume and not a.reanalyze else 'analyze',largest['id'])[0]
    job=command('jobs','show',analysis['job_id'])[0]
    assert job['result']['analysis_limited'] is True and job['result']['sections'] is None
    assert analysis['analysis']['sha256']==largest['sha256'] and analysis['analysis']['size']==largest['size']
    assert analysis['analysis']['warnings'] and not analysis['analysis']['sections']
    assert any('first 1 MiB' in w for w in analysis['analysis']['warnings'])
    assert command('reverse','metadata',largest['id'])[0]['id']==analysis['id']
    checks.append('Real largest-file Job Engine metadata analysis persists after process restart; deep parsing explicitly deferred')
    if os.name!='nt':
        original=out/f'benign-{sizes[-1]}MiB{sample.suffix or ".elf"}'
        cancel=out/'owned-cancel-fixture.elf'
        if not cancel.exists(): os.link(original,cancel)
        before=command('reverse','list','--project',project['id'])[0]
        process=subprocess.Popen([str(binary),'--json','reverse','import',str(cancel),'--project',project['id'],'--authorized','--progress'],env=env,text=True,encoding='utf-8',stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        first=process.stderr.readline();assert 'Copying and hashing' in first
        process.send_signal(signal.SIGINT);stdout,stderr=process.communicate(timeout=30)
        assert process.returncode and 'CANCELLED' in stderr
        assert len(command('reverse','list','--project',project['id'])[0])==len(before)
        assert not list((out/'workspace/imports').glob('.import-*'))
        calls.append({'arguments':['reverse','import','<owned cancel fixture>','--progress'],'exit_code':process.returncode,'expected':'CANCELLED after SIGINT','timestamp':dt.datetime.now(dt.timezone.utc).isoformat()})
        (out/'commands.json').write_text(json.dumps(calls,indent=2))
        checks.append('Actual Linux CLI SIGINT cancels running streaming copy with structured CANCELLED error and removes staging without new records')
    result={'status':'passed','platform':platform.platform(),'timestamp':dt.datetime.now(dt.timezone.utc).isoformat(),'binary_sha256':digest(binary),'checks':checks,'commands':len(calls),'files':[{'id':f['id'],'size':f['size'],'sha256':f['sha256']} for f in imports],'limits':'No binary execution. Complete bytes/hash/entropy; strings/hex bounded to first 1 MiB; no deep sections/imports/decompiler claim.'}
    (out/'results.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))

if __name__=='__main__': main()
