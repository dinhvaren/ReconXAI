-- Coordination metadata only. analysis_jobs remains the sole execution queue.
CREATE TABLE research_runtimes (
 session_id TEXT PRIMARY KEY REFERENCES research_sessions(id),
 fingerprint TEXT NOT NULL, limits_json TEXT NOT NULL CHECK(json_valid(limits_json)),
 status TEXT NOT NULL CHECK(status IN ('running','pause_requested','paused','completed','failed','cancelled')),
 created_at TEXT NOT NULL, deadline_ms INTEGER NOT NULL,
 invocations INTEGER NOT NULL DEFAULT 0 CHECK(invocations>=0),
 stop_reason TEXT, revision INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE research_agent_assignments (
 session_id TEXT NOT NULL REFERENCES research_runtimes(session_id),
 agent_id TEXT NOT NULL, descriptor_json TEXT NOT NULL CHECK(json_valid(descriptor_json)),
 PRIMARY KEY(session_id,agent_id)
);
CREATE TABLE research_agent_tasks (
 id TEXT PRIMARY KEY, session_id TEXT NOT NULL REFERENCES research_runtimes(session_id),
 ordinal INTEGER NOT NULL, agent_id TEXT NOT NULL, capability_id TEXT NOT NULL,
 state TEXT NOT NULL CHECK(state IN ('PENDING','READY','RUNNING','PAUSE_REQUESTED','PAUSED','RETRY_WAIT','COMPLETED','FAILED','CANCELLED','BLOCKED')),
 job_id TEXT NOT NULL UNIQUE REFERENCES analysis_jobs(id),
 attempt INTEGER NOT NULL DEFAULT 0 CHECK(attempt BETWEEN 0 AND 2),
 retry_at_ms INTEGER, error_code TEXT,
 UNIQUE(session_id,id), UNIQUE(session_id,ordinal),
 FOREIGN KEY(session_id,agent_id) REFERENCES research_agent_assignments(session_id,agent_id)
);
CREATE TABLE research_task_dependencies (
 session_id TEXT NOT NULL, task_id TEXT NOT NULL, dependency_id TEXT NOT NULL,
 PRIMARY KEY(task_id,dependency_id), CHECK(task_id<>dependency_id),
 FOREIGN KEY(session_id,task_id) REFERENCES research_agent_tasks(session_id,id),
 FOREIGN KEY(session_id,dependency_id) REFERENCES research_agent_tasks(session_id,id)
);
CREATE TABLE research_task_attempts (
 task_id TEXT NOT NULL REFERENCES research_agent_tasks(id), attempt INTEGER NOT NULL,
 job_id TEXT NOT NULL UNIQUE REFERENCES analysis_jobs(id),
 PRIMARY KEY(task_id,attempt)
);
CREATE TABLE research_agent_messages (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 session_id TEXT NOT NULL REFERENCES research_runtimes(session_id),
 task_id TEXT, agent_id TEXT NOT NULL, correlation_id TEXT NOT NULL,
 message_type TEXT NOT NULL, created_at TEXT NOT NULL,
 payload_json TEXT NOT NULL CHECK(json_valid(payload_json)),
 evidence_json TEXT NOT NULL CHECK(json_valid(evidence_json)),
 UNIQUE(session_id,correlation_id,message_type),
 FOREIGN KEY(session_id,task_id) REFERENCES research_agent_tasks(session_id,id),
 FOREIGN KEY(session_id,agent_id) REFERENCES research_agent_assignments(session_id,agent_id)
);
CREATE INDEX research_agent_messages_session ON research_agent_messages(session_id,id);
CREATE INDEX research_agent_tasks_session ON research_agent_tasks(session_id,state);
CREATE TABLE research_task_transitions (
 id INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT NOT NULL, task_id TEXT NOT NULL,
 previous_state TEXT, next_state TEXT NOT NULL, job_id TEXT NOT NULL REFERENCES analysis_jobs(id),
 attempt INTEGER NOT NULL, created_at TEXT NOT NULL,
 FOREIGN KEY(session_id,task_id) REFERENCES research_agent_tasks(session_id,id)
);
CREATE INDEX research_task_transitions_session ON research_task_transitions(session_id,id);
CREATE TRIGGER research_task_state_guard BEFORE UPDATE OF state ON research_agent_tasks
WHEN NEW.state<>OLD.state AND NOT (
 (OLD.state='PENDING' AND NEW.state IN ('READY','PAUSED','CANCELLED','BLOCKED')) OR
 (OLD.state='READY' AND NEW.state IN ('RUNNING','PAUSED','CANCELLED','BLOCKED')) OR
 (OLD.state='RUNNING' AND NEW.state IN ('PAUSE_REQUESTED','COMPLETED','FAILED','CANCELLED')) OR
 (OLD.state='PAUSE_REQUESTED' AND NEW.state IN ('COMPLETED','FAILED','CANCELLED')) OR
 (OLD.state='PAUSED' AND NEW.state IN ('PENDING','CANCELLED','BLOCKED')) OR
 (OLD.state='RETRY_WAIT' AND NEW.state IN ('READY','PAUSED','CANCELLED','BLOCKED')) OR
 (OLD.state IN ('FAILED','BLOCKED','CANCELLED') AND NEW.state='RETRY_WAIT' AND NEW.attempt=OLD.attempt+1 AND NEW.job_id<>OLD.job_id) OR
 (OLD.state='BLOCKED' AND NEW.state='CANCELLED')
) BEGIN SELECT RAISE(ABORT,'Illegal research task transition'); END;
CREATE TRIGGER research_task_created AFTER INSERT ON research_agent_tasks BEGIN
 INSERT INTO research_task_transitions(session_id,task_id,previous_state,next_state,job_id,attempt,created_at)
 VALUES(NEW.session_id,NEW.id,NULL,NEW.state,NEW.job_id,NEW.attempt,strftime('%Y-%m-%dT%H:%M:%fZ','now'));
END;
CREATE TRIGGER research_task_transitioned AFTER UPDATE OF state ON research_agent_tasks WHEN NEW.state<>OLD.state BEGIN
 INSERT INTO research_task_transitions(session_id,task_id,previous_state,next_state,job_id,attempt,created_at)
 VALUES(NEW.session_id,NEW.id,OLD.state,NEW.state,NEW.job_id,NEW.attempt,strftime('%Y-%m-%dT%H:%M:%fZ','now'));
END;
