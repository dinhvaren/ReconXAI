"""Verify read-only research graphs using a copy of an owned recon_tools_smoke workspace.

Requires an existing completed fixture. Does not execute new tools or AI requests
and never modifies the source workspace. The destination must not already exist.
"""
from pathlib import Path
import argparse,datetime as dt,hashlib,json,os,shutil,sqlite3,subprocess
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--source',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
binary=a.binary.resolve(strict=True);source=a.source.resolve(strict=True);out=a.output.resolve();out.mkdir(exist_ok=False,parents=True);workspace=out/'workspace';workspace.mkdir();
if os.name=='posix':os.chmod(out,0o700);os.chmod(workspace,0o700)
original=next(source.glob('*.db'));db=workspace/'reconx.db'
with sqlite3.connect(original.as_uri()+'?mode=ro',uri=True) as old,sqlite3.connect(db) as copy:old.backup(copy)
for name in ['evidence','reports']:
 if (source/name).is_dir():shutil.copytree(source/name,workspace/name)
with sqlite3.connect(db) as c:
 before_version=c.execute('PRAGMA user_version').fetchone()[0]
 session=c.execute('SELECT id FROM research_sessions ORDER BY created_at DESC LIMIT 1').fetchone()[0]
 jobs_before=c.execute('SELECT id,status,result_json FROM analysis_jobs ORDER BY id').fetchall()
 target=c.execute('SELECT target_id FROM research_sessions WHERE id=?',[session]).fetchone()[0]
env={**os.environ,'RECONX_DATA_DIR':str(workspace)};commands=[]
def call(*args):
 r=subprocess.run([str(binary),'--json',*map(str,args)],env=env,capture_output=True,text=True,encoding='utf-8',timeout=60);commands.append({'args':list(map(str,args)),'exit_code':r.returncode});assert not r.returncode,(r.stderr+r.stdout)[:1200];return json.loads(r.stdout)
g=call('graph','nodes',session);assert g['total_nodes']>=10
assert len([n for n in g['nodes'] if n['kind']=='url'])==1
assert any(e['relation']=='SUPPORTED_BY' for e in g['edges'])
again=call('graph','nodes',session);assert again['revision']==g['revision'] and again['source_sha256']==g['source_sha256']
assert [n['id'] for n in again['nodes']]==[n['id'] for n in g['nodes']]
e=next(n for n in g['nodes'] if n['kind']=='evidence');filtered=call('graph','nodes',session,'--kind','evidence','--search',e['source_id']);assert len(filtered['nodes'])==1
assert call('evidence','verify',e['source_id'])['integrity']=='matched'
small=call('graph','nodes',session,'--limit','2');assert len(small['nodes'])==2 and small['next_offset']==2
assert call('graph','nodes',session,'--graph','asset')['matched_nodes']==3
call('graph','export',session,'--output',out/'graph.json');exported=json.loads((out/'graph.json').read_text(encoding='utf-8'));ids={n['id'] for n in exported['nodes']};assert all(x['source'] in ids and x['target'] in ids for x in exported['edges'])
assert not any('source_path' in n['attributes'] for n in exported['nodes'])
with sqlite3.connect(db) as c:
 assert c.execute('PRAGMA user_version').fetchone()[0]==19
 assert c.execute('SELECT id,status,result_json FROM analysis_jobs ORDER BY id').fetchall()==jobs_before
 assert c.execute('PRAGMA integrity_check').fetchone()[0]=='ok' and not c.execute('PRAGMA foreign_key_check').fetchall()
 assert c.execute('SELECT count(*) FROM research_graph_nodes WHERE session_id=?',[session]).fetchone()[0]==g['total_nodes']
 assert c.execute('SELECT target_id FROM research_sessions WHERE id=?',[session]).fetchone()[0]==target
# Exercise actual target removal only in the copied owned fixture. Stored jobs,
# results, observations and evidence remain readable, with an unavailable target.
call('target','remove',target)
history=call('graph','nodes',session)
assert {n['id'] for n in history['nodes']}=={n['id'] for n in g['nodes']}
assert history['total_edges']==g['total_edges']
assert next(n for n in history['nodes'] if n['kind']=='target')['attributes']['source_state']=='removed'
assert call('evidence','verify',e['source_id'])['integrity']=='matched'
assert call('graph','nodes',session)['revision']==history['revision']
call('graph','export',session,'--output',out/'historical-graph.json')
with sqlite3.connect(db) as c:
 assert c.execute('SELECT id,status,result_json FROM analysis_jobs ORDER BY id').fetchall()==jobs_before
 assert c.execute('SELECT target_id FROM research_sessions WHERE id=?',[session]).fetchone()[0] is None
 assert c.execute('PRAGMA integrity_check').fetchone()[0]=='ok' and not c.execute('PRAGMA foreign_key_check').fetchall()
result={'status':'passed','timestamp':dt.datetime.now(dt.timezone.utc).isoformat(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'old_schema':before_version,'new_schema':19,'session_id':session,'nodes':g['total_nodes'],'edges':g['total_edges'],'checks':['Consistent SQLite backup of an owned real HTTPX workspace migrates to schema019 with original jobs/targets/results unchanged','Stored graph projects real observations, hashed evidence and directed relationships without executing another job','CLI filtering/pagination, repeated-process persistence/idempotent revision, integrity check and complete graph export pass','Removing the copied fixture target preserves graph IDs, observed assets, jobs, evidence integrity and historical export after restart'],'commands':commands,'network':'No new tool/AI execution; graph reads stored fixture observations'}
(out/'results.json').write_text(json.dumps(result,indent=2),encoding='utf-8');print(json.dumps(result,indent=2))
