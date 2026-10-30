"""Real offline DAG CLI acceptance. Owned-loopback MOCK planning, never cloud AI."""
import argparse
import datetime as dt
import hashlib
import json
import os
import platform
import subprocess
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--sample', type=Path, help='Optional owned PE/ELF; never executed')
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    env = {**os.environ, 'RECONX_DATA_DIR': str(out / 'workspace')}
    calls, checks, model_calls = [], [], []

    class Mock(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_POST(self):
            assert self.path == '/v1/chat/completions'
            request = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            assert 'Return strict JSON only' in str(request)
            model_calls.append({'model': request['model'], 'path': self.path})
            proposal = {'steps': [
                {'capability_id': 'local.metadata', 'reason': 'Inspect the benign imported file'},
                {'capability_id': 'local.strings', 'reason': 'Extract bounded local strings'},
            ]}
            body = json.dumps({'choices': [{'message': {'content': json.dumps(proposal)}}],
                               'usage': {'prompt_tokens': 17, 'completion_tokens': 11}}).encode()
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            self.wfile.write(body)

    server = ThreadingHTTPServer(('127.0.0.1', 0), Mock)
    worker = threading.Thread(target=server.serve_forever, daemon=True)
    worker.start()

    def data(name, value):
        path = out / name
        path.write_text(json.dumps(value), encoding='utf-8')
        return path

    def command(*values, fail=False, timeout=90):
        arguments = list(map(str, values))
        started = dt.datetime.now(dt.timezone.utc).isoformat()
        result = subprocess.run([str(binary), '--json', *arguments], env=env,
                                capture_output=True, text=True, encoding='utf-8', timeout=timeout)
        calls.append({'arguments': arguments, 'timestamp': started, 'exit_code': result.returncode})
        if fail:
            assert result.returncode != 0, f'Expected rejection: {arguments}'
            return
        assert result.returncode == 0, result.stderr[:2000]
        return json.loads(result.stdout)

    try:
        project = command('project', 'create', 'Offline agent acceptance')
        sample = out / 'benign.bin'
        sample.write_bytes(args.sample.resolve(strict=True).read_bytes() if args.sample else
                           b'RECONX_BENIGN_AGENT_ACCEPTANCE\0read-only fixture\0')
        imported = command('reverse', 'import', sample, '--project', project['id'], '--authorized')
        assert imported['sha256'] == hashlib.sha256(sample.read_bytes()).hexdigest()
        if args.sample:
            assert imported['format'] in ('Pe', 'Elf'), imported['format']
        command('workspace', 'select', '--project', project['id'], '--target', imported['target_id'])
        command('ai', 'profiles', 'create', '--vendor', 'lmstudio', '--name', 'Owned MOCK planner',
                '--model', 'agent-fixture', '--base-url', f'http://127.0.0.1:{server.server_port}/v1', '--default')
        provider = command('settings', 'show')['default_provider_id']
        preview = command('research', 'preview', imported['target_id'], '--provider', provider)
        review = {k: preview[k] for k in ('request', 'context', 'fingerprint')}
        review['consent'] = False
        command('research', 'plan', data('declined-plan.json', review), fail=True)
        assert not model_calls
        review['consent'] = True
        session = command('research', 'plan', data('approved-plan.json', review))
        session_id = session['plan']['id']
        assert len(model_calls) == 1
        dag = command('research', 'dag-preview', session_id)
        approval = {'session_id': session_id, 'limits': dag['limits'],
                    'fingerprint': dag['fingerprint'], 'approved': False}
        command('research', 'dag-start', data('declined-dag.json', approval), fail=True)
        stale = {**approval, 'approved': True, 'limits': {**dag['limits'], 'concurrency': 1}}
        command('research', 'dag-start', data('stale-dag.json', stale), fail=True)
        assert not command('jobs', 'list')
        approval['approved'] = True
        runtime = command('research', 'dag-start', data('approved-dag.json', approval))
        original_jobs = {task['job_id'] for task in runtime['tasks']}
        assert len(original_jobs) == 5
        assert command('research', 'pause', session_id)['status'] == 'paused'
        before = time.monotonic()
        command('jobs', 'run', timeout=15)
        assert time.monotonic() - before < 15
        paused = command('agents', 'status', session_id)
        assert paused['invocations'] == 0 and all(t['state'] == 'PAUSED' for t in paused['tasks'])
        # Each command opens the existing database in a new process: no hidden in-memory state.
        assert command('workspace', 'show')['target_id'] == imported['target_id']
        command('research', 'resume', session_id)
        command('jobs', 'run', '--concurrency', '2')
        tasks = command('workflow', 'tasks', session_id)
        runtime = command('agents', 'status', session_id)
        assert tasks == runtime['tasks']
        assert runtime['status'] == 'completed', runtime
        assert runtime['invocations'] == 5
        assert {t['job_id'] for t in runtime['tasks']} == original_jobs
        assert all(t['state'] == 'COMPLETED' and t['attempt'] == 0 for t in runtime['tasks'])
        assert runtime['model_requests_by_runtime'] == runtime['network_requests_by_runtime'] == 0
        assert runtime['runtime_cost'] is None and len(model_calls) == 1
        session = command('research', 'show', session_id)
        assert session['status'] == 'completed' and session['runtime_enabled']
        assert len(session['jobs']) == len(session['evidence']) == 5
        jobs = {j['id']: j for j in session['jobs']}
        for task in runtime['tasks']:
            for dependency in task['dependencies']:
                parent = next(t for t in runtime['tasks'] if t['id'] == dependency)
                assert jobs[parent['job_id']]['finished_at'] <= jobs[task['job_id']]['started_at']
        for evidence in session['evidence']:
            command('evidence', 'verify', evidence['id'])
        checks.append('Explicit MOCK planning consent and immutable DAG approval; five real dependent jobs, integrity validation and snapshot')
        checks.append('Pause persists across CLI restarts; jobs run returns while paused; resume preserves jobs and executes each once')
        assert len(session['reports']) == 1
        report = command('report', 'show', session['reports'][0]['id'])
        for fmt, suffix in [('json', 'json'), ('pdf', 'pdf'), ('html', 'html'), ('markdown', 'md')]:
            destination = out / f'agent-report.{suffix}'
            command('report', 'export', report['report']['id'], '--format', fmt,
                    '--output', destination, '--reviewed-sha256', report['report']['sha256'])
            content = destination.read_bytes()
            assert content
            if fmt == 'json':
                json.loads(content)
            if fmt == 'pdf':
                assert content.startswith(b'%PDF-')
        checks.append('Real Report Engine output independently opens as JSON/PDF and exports all four formats')
        command('workflow', 'graph', session_id)
        command('graph', 'export', session_id, '--output', out / 'graph.json')
        graph = json.loads((out / 'graph.json').read_text(encoding='utf-8'))
        assert graph['session_id'] == session_id
        nodes = graph['nodes']
        kinds = {n['kind'] for n in nodes}
        assert {'agent', 'agent_task', 'dag', 'state_transition', 'evidence'}.issubset(kinds), kinds
        assert len([n for n in nodes if n['kind'] == 'agent']) == 6
        assert len([n for n in nodes if n['kind'] == 'agent_task']) == 5
        assert any(e['relation'] == 'DEPENDS_ON' for e in graph['edges'])
        assert command('research', 'show', session_id)['reports'][0]['id'] == report['report']['id']
        command('research', 'dag-start', out / 'approved-dag.json', fail=True)
        assert len(command('jobs', 'list')) == 5 and len(model_calls) == 1
        checks.append('Persistent real graph provenance, task dependencies, assignments and transitions; duplicate execution rejected')
        data('runtime.json', runtime)
        result = {'status': 'passed', 'platform': platform.platform(), 'timestamp': dt.datetime.now(dt.timezone.utc).isoformat(),
                  'checks': checks, 'commands': len(calls), 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
                  'model': 'MOCK_VERIFIED owned loopback, one planning request; no cloud verification',
                  'session_id': session_id, 'schema': 20, 'runtime_network_requests': 0}
        data('results.json', result)
        print(json.dumps(result, indent=2))
    finally:
        data('commands.json', calls)
        data('model-calls.json', model_calls)
        server.shutdown()
        server.server_close()
        worker.join()


if __name__ == '__main__':
    main()
