"""Actual shared-Core CLI research acceptance; owned loopback + mock AI only."""
import argparse
import datetime as dt
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import platform
import subprocess
import threading


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--sample', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    binary, sample, out = a.binary.resolve(strict=True), a.sample.resolve(strict=True), a.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    env = {**os.environ, 'RECONX_DATA_DIR': str(out/'workspace')}
    calls, checks, ai_calls, http_calls = [], [], [], []

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass
        def do_HEAD(self):
            http_calls.append({'method':'HEAD','path':self.path})
            self.send_response(200); self.send_header('Content-Length','0'); self.end_headers()
        def do_POST(self):
            body=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            ai_calls.append(body)
            prompt='\n'.join(m.get('content','') for m in body['messages'])
            content=json.dumps({'steps':[{'capability_id':'local.scope_review','reason':'Review authorization'},{'capability_id':'local.http_head','reason':'Observe one approved owned fixture'}]}) if 'Return strict JSON only' in prompt else 'Mock interpretation of actual cited evidence; independent verification required.'
            reply=json.dumps({'choices':[{'message':{'content':content}}],'usage':{'prompt_tokens':17,'completion_tokens':11}}).encode()
            self.send_response(200); self.send_header('Content-Type','application/json'); self.send_header('Content-Length',str(len(reply))); self.end_headers(); self.wfile.write(reply)
    server=ThreadingHTTPServer(('127.0.0.1',0),Handler)
    thread=threading.Thread(target=server.serve_forever,daemon=True); thread.start()
    base=f'http://127.0.0.1:{server.server_port}'

    def command(*args, fail=False):
        r=subprocess.run([str(binary),'--json',*map(str,args)],env=env,capture_output=True,text=True,encoding='utf-8',timeout=90)
        calls.append({'arguments':list(map(str,args)),'exit_code':r.returncode,'timestamp':dt.datetime.now(dt.timezone.utc).isoformat()})
        if fail:
            assert r.returncode != 0, 'Expected policy rejection'; return
        if r.returncode: raise RuntimeError(r.stderr[:1500])
        return json.loads(r.stdout)
    def data(name,value):
        f=out/name; f.write_text(json.dumps(value),encoding='utf-8'); return f
    try:
        project=command('project','create','Research Demo')
        settings=command('settings','show'); settings['security_mode']='lab_research'
        command('settings','apply',data('settings.json',settings))
        target=command('target','add','--project',project['id'],'--type','url','--name','Owned CLI fixture','--value',base+'/','--authorized','--mode','lab_research')
        command('workspace','select','--project',project['id'],'--target',target['id'])
        assert command('workspace','show')['target_id']==target['id']
        command('ai','profiles','create','--vendor','lmstudio','--name','Local deterministic mock','--model','research-fixture','--base-url',base+'/v1','--default')
        provider=command('settings','show')['default_provider_id']
        preview=command('research','preview',target['id'],'--provider',provider)
        review={'request':preview['request'],'context':preview['context'],'fingerprint':preview['fingerprint'],'consent':False}
        command('research','plan',data('declined.json',review),fail=True)
        assert not ai_calls and not http_calls
        review['consent']=True
        session=command('research','plan',data('approved.json',review))
        assert session['status']=='awaiting_approval' and not http_calls
        command('research','approve',session['plan']['id'],'--fingerprint',preview['fingerprint'],'--approve',fail=True)
        assert not command('jobs','list')
        command('research','approve',session['plan']['id'],'--fingerprint',preview['fingerprint'],'--approve','--approve-network')
        command('jobs','run')
        session=command('research','show',session['plan']['id']); assert session['status']=='completed'
        assert len(http_calls)==1 and len(session['evidence'])>=2
        checks.append('CLI MOCK AI typed planning + separate approved real HEAD + atomic policy rejection + persisted evidence')
        for evidence in session['evidence']: command('evidence','verify',evidence['id'])
        context=command('research','evidence',session['plan']['id'])
        command('research','interpret',data('interpret.json',{'id':session['plan']['id'],'provider_id':provider,'gateway':base+'/v1','model':'research-fixture','preview':context,'consent':True}))
        details=out/'finding.txt'; details.write_text('Manual review note; not a vulnerability claim.\nAuthorization: Bearer synthetic-research-export-secret',encoding='utf-8')
        finding=command('research','finding',session['plan']['id'],'--title','Review local evidence','--details-file',details)
        assert finding['input']['status']=='needs_verification'
        report=command('research','report',session['plan']['id'])
        for fmt, ext in [('markdown','md'),('json','json'),('html','html'),('pdf','pdf')]:
            output=out/('research-report.'+ext)
            command('report','export',report['report']['id'],'--format',fmt,'--output',output,'--reviewed-sha256',report['report']['sha256'])
            blob=output.read_bytes(); assert blob and b'synthetic-research-export-secret' not in blob
            if fmt=='pdf': assert blob.startswith(b'%PDF-')
            if fmt=='json': json.loads(blob)
        checks.append('AI interpretation consent + Needs Verification finding + four redacted report formats')
        imported=command('reverse','import',sample,'--project',project['id'],'--authorized')
        assert imported['sha256']==hashlib.sha256(sample.read_bytes()).hexdigest()
        command('workspace','select','--project',project['id'],'--target',imported['target_id'])
        p=command('research','preview',imported['target_id'],'--mode','manual')
        f=command('research','plan',data('file-plan.json',{'request':p['request'],'context':p['context'],'fingerprint':p['fingerprint'],'consent':False}))
        command('research','approve',f['plan']['id'],'--fingerprint',p['fingerprint'],'--approve')
        command('jobs','run')
        file_session=command('research','show',f['plan']['id'])
        assert file_session['status']=='completed' and len(file_session['evidence'])==3
        assert len(http_calls)==1
        assert command('workspace','show')['target_id']==imported['target_id']
        assert command('research','show',session['plan']['id'])['reports'][0]['id']==report['report']['id']
        checks.append('Real benign PE/ELF static session; independent hash; persistent selection across separate CLI processes')
        result={'status':'passed','platform':platform.platform(),'timestamp':dt.datetime.now(dt.timezone.utc).isoformat(),'checks':checks,'command_count':len(calls),'ai':'MOCK_VERIFIED only; no cloud request','sample_sha256':imported['sha256'],'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}
        (out/'results.json').write_text(json.dumps(result,indent=2)); print(json.dumps(result,indent=2))
    finally:
        server.shutdown(); server.server_close(); thread.join()
        (out/'commands.json').write_text(json.dumps(calls,indent=2))

if __name__=='__main__': main()
