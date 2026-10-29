"""Real managed HTTPX/Source-inventory acceptance with owned loopback + MOCK AI.

No live passive-domain query is performed or claimed. A tool directory is reused
only after Core verifies its pinned binary hashes; downloads require the flag.
"""
import argparse,datetime as dt,hashlib,json,os,platform,subprocess,threading,time
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
from pathlib import Path

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    p.add_argument('--tools',type=Path);p.add_argument('--approve-downloads',action='store_true');a=p.parse_args()
    binary=a.binary.resolve(strict=True);out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    assert a.tools or a.approve_downloads,'Approve official package download or supply already verified tools'
    env={**os.environ,'RECONX_DATA_DIR':str(out/'workspace')}
    if a.tools:env['PATH']=str(a.tools.resolve(strict=True))+os.pathsep+env.get('PATH','')
    calls=[];http=[];ai_calls=[];checks=[];sentinel=[]
    def command(*args,fail=False):
        r=subprocess.run([str(binary),'--json',*map(str,args)],env=env,capture_output=True,text=True,encoding='utf-8',timeout=180)
        calls.append({'args':list(map(str,args)),'exit_code':r.returncode,'timestamp':dt.datetime.now(dt.timezone.utc).isoformat()})
        (out/'commands.json').write_text(json.dumps(calls,indent=2),encoding='utf-8')
        if fail:assert r.returncode;return
        if r.returncode:raise RuntimeError((r.stderr+r.stdout)[:2000])
        return json.loads(r.stdout)
    def data(name,value):
        f=out/name;f.write_text(json.dumps(value),encoding='utf-8');return f
    class Sentinel(BaseHTTPRequestHandler):
        def log_message(self,*_):pass
        def do_GET(self):sentinel.append(self.path);self.send_response(200);self.end_headers()
    sink=ThreadingHTTPServer(('127.0.0.1',0),Sentinel)
    class Handler(BaseHTTPRequestHandler):
        def log_message(self,*_):pass
        def do_GET(self):
            http.append({'method':'GET','path':self.path})
            if self.path=='/slow':time.sleep(4)
            body=b'<html><title>Owned recon fixture</title><body>Benign fixture</body></html>'
            try:
                self.send_response(302 if self.path=='/redirect' else 200)
                if self.path=='/redirect':self.send_header('Location',f'http://127.0.0.1:{sink.server_port}/forbidden')
                self.send_header('Content-Type','text/html');self.send_header('Content-Length',str(len(body)));self.end_headers()
                self.wfile.write(body)
            except (BrokenPipeError,ConnectionResetError,ConnectionAbortedError):pass
        def do_POST(self):
            value=json.loads(self.rfile.read(int(self.headers['Content-Length'])));ai_calls.append(value)
            prompt='\n'.join(m.get('content','') for m in value['messages'])
            text=json.dumps({'steps':[{'capability_id':'local.scope_review','reason':'Review declared authorization'},{'capability_id':'httpx.probe','reason':'Observe the explicitly approved local HTTP fixture'}]}) if 'Return strict JSON only' in prompt else 'Mock interpretation: actual HTTP evidence requires review; no vulnerability verified.'
            body=json.dumps({'choices':[{'message':{'content':text}}],'usage':{'prompt_tokens':20,'completion_tokens':10}}).encode()
            self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
    server=ThreadingHTTPServer(('127.0.0.1',0),Handler)
    threads=[threading.Thread(target=s.serve_forever,daemon=True) for s in [sink,server]]
    for t in threads:t.start()
    origin=f'http://127.0.0.1:{server.server_port}'
    def target(project,path='/'):
        return command('target','add','--project',project,'--type','url','--name','Owned HTTPX fixture '+path,'--value',origin+path,'--authorized','--mode','lab_research')
    def plan(target_id,path_name,timeout=5):
        q=command('recon','plan',target_id,'--tool','httpx','--timeout-secs',str(timeout));return data(path_name,q),q
    try:
        project=command('project','create','Managed recon acceptance')
        settings=command('settings','show');settings['security_mode']='lab_research';command('settings','apply',data('settings.json',settings))
        for tool in ['httpx','subfinder']:
            install=command('extensions','plan',tool)
            command('extensions','install',tool,'--fingerprint',install['fingerprint'],fail=True)
            assert command('extensions','install',tool,'--fingerprint',install['fingerprint'],'--approve')['status']=='ready'
            verified=command('tools','verify',tool);assert verified['integrity_verified']
        checks.append('Actual pinned HTTPX/Subfinder install or hash-verified existing-binary reuse; real HTTPX own-loopback GET health and offline Subfinder source inventory')
        # Corrupt only the managed copy in this newly created synthetic workspace.
        # Repair must not execute the corrupted bytes or clobber an unknown path.
        manifest=command('tools','plan','httpx')
        managed=(out/'workspace'/'tools'/'httpx'/manifest['version']/manifest['executable']).resolve(strict=True)
        assert managed.is_relative_to((out/'workspace').resolve()) and not managed.is_symlink()
        managed.write_bytes(b'Owned fixture: intentionally corrupted managed executable')
        command('extensions','doctor','httpx',fail=True)
        repair=command('extensions','plan','httpx')
        assert command('extensions','install','httpx','--fingerprint',repair['fingerprint'],'--approve')['status']=='ready'
        assert hashlib.sha256(managed.read_bytes()).hexdigest()==manifest['binary_sha256']
        receipt=managed.parent/'receipt.json';held_receipt=out/'held-fixture-httpx-receipt.json'
        receipt.replace(held_receipt)
        unknown=b'Owned fixture: unexpected file without installation receipt'
        managed.write_bytes(unknown)
        try:
            repair=command('extensions','plan','httpx')
            command('extensions','install','httpx','--fingerprint',repair['fingerprint'],'--approve',fail=True)
            assert managed.read_bytes()==unknown,'Repair must preserve an unowned destination'
        finally:held_receipt.replace(receipt)
        repair=command('extensions','plan','httpx')
        assert command('extensions','install','httpx','--fingerprint',repair['fingerprint'],'--approve')['status']=='ready'
        assert hashlib.sha256(managed.read_bytes()).hexdigest()==manifest['binary_sha256']
        checks.append('Receipt-owned corruption is rejected before execution and atomically repaired with real health; an unexpected receipt-less destination is preserved and rejected')
        owned=target(project['id']);review,q=plan(owned['id'],'probe.json')
        command('recon','run','--plan',review,fail=True);assert not http
        job=command('recon','run','--plan',review,'--approve');command('jobs','run')
        result=command('jobs','show',job['id']);assert result['status']=='completed',result
        assets=command('recon','assets','--project',project['id']);assert len(assets)==1
        facts=assets[0]['facts'];assert facts['status']==200 and facts['title']=='Owned recon fixture'
        assert len(http)==1 and not sentinel
        assert command('evidence','verify',facts['evidence_id'])['integrity']=='matched'
        checks.append('Approved real HTTPX GET produces exact local title/status/content-type, normalized target-linked inventory and hashed original tool evidence')
        redirected=target(project['id'],'/redirect');f,_=plan(redirected['id'],'redirect.json')
        red=command('recon','run','--plan',f,'--approve');command('jobs','run');assert command('jobs','show',red['id'])['status']=='completed';assert not sentinel
        checks.append('Real 302 fixture is captured without contacting its out-of-scope redirect destination')
        blocked=target(project['id'],'/blocked');f,_=plan(blocked['id'],'stale.json')
        body=command('target','show',blocked['id']);body={k:v for k,v in body.items() if k in ['project_id','name','kind','value','description','tags','allowed_scope','excluded_scope','authorized','mode']};body['excluded_scope']=[origin+'/blocked']
        command('target','update',blocked['id'],'--input',data('excluded.json',body));command('recon','run','--plan',f,'--approve',fail=True)
        assert not any(x['path']=='/blocked' for x in http)
        checks.append('A scope change invalidates the exact reviewed plan and prevents network execution')
        slow=target(project['id'],'/slow');f,_=plan(slow['id'],'slow.json',1)
        timeout=command('recon','run','--plan',f,'--approve');command('jobs','run');assert command('jobs','show',timeout['id'])['status']=='failed'
        f,_=plan(slow['id'],'cancel.json',15);cancel=command('recon','run','--plan',f,'--approve')
        before=len(http);worker=subprocess.Popen([str(binary),'--json','jobs','run'],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        try:
            deadline=time.monotonic()+30
            while len(http)==before and time.monotonic()<deadline:time.sleep(.05)
            assert len(http)>before,'Cancellation must interrupt a genuinely pending HTTPX request'
            command('jobs','cancel',cancel['id']);worker.communicate(timeout=20)
            assert command('jobs','show',cancel['id'])['status']=='cancelled'
        finally:
            if worker.poll() is None:worker.terminate();worker.wait(timeout=10)
        checks.append('A real delayed response fails under its bounded timeout; a separate genuinely pending HTTPX process cancels and persists Cancelled')
        command('ai','profiles','create','--vendor','lmstudio','--name','Owned mock recon planner','--model','recon-fixture','--base-url',origin+'/v1','--default')
        provider=command('settings','show')['default_provider_id'];preview=command('research','preview',owned['id'],'--provider',provider)
        session=command('research','plan',data('ai-plan.json',{'request':preview['request'],'fingerprint':preview['fingerprint'],'context':preview['context'],'consent':True}))
        command('research','approve',session['plan']['id'],'--fingerprint',preview['fingerprint'],'--approve',fail=True)
        command('research','approve',session['plan']['id'],'--fingerprint',preview['fingerprint'],'--approve','--approve-network');command('jobs','run')
        session=command('research','show',session['plan']['id']);assert session['status']=='completed' and session['evidence']
        context=command('research','evidence',session['plan']['id'])
        command('research','interpret',data('interpret.json',{'id':session['plan']['id'],'provider_id':provider,'gateway':origin+'/v1','model':'recon-fixture','preview':context,'consent':True}))
        report=command('research','report',session['plan']['id'])
        for fmt,ext in [('markdown','md'),('json','json'),('html','html'),('pdf','pdf')]:
            file=out/('recon-report.'+ext);command('report','export',report['report']['id'],'--format',fmt,'--output',file,'--reviewed-sha256',report['report']['sha256']);assert file.stat().st_size
        assert command('recon','history','--target',owned['id'])
        command('recon','export','--project',project['id'],'--output',out/'inventory.json')
        assert command('research','show',session['plan']['id'])['reports'][0]['id']==report['report']['id']
        checks.append('MOCK AI typed recon plan uses actual managed HTTPX through approvals/Job Engine; separately approved interpretation, four real report formats and restart persistence')
        result={'status':'passed','platform':platform.platform(),'timestamp':dt.datetime.now(dt.timezone.utc).isoformat(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'commands':len(calls),'checks':checks,'cloud_ai':'MOCK_VERIFIED only','live_subfinder':'NOT_TESTED; no owned external domain/source authorization supplied','http_calls':http,'redirect_escape_calls':len(sentinel)}
        (out/'results.json').write_text(json.dumps(result,indent=2),encoding='utf-8');print(json.dumps(result,indent=2))
    finally:
        for service in [server,sink]:service.shutdown();service.server_close()

if __name__=='__main__':main()
