//! Bounded, offline consistency observations. Never a security verdict or network client.
use super::*;

const MAX_CHECKS: usize = 10_000;
const MAX_OBSERVATIONS: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaObservation {
    pub classification: String,
    pub rule: String,
    pub message: String,
    /// Pointer to a HAR field; instance_pointer addresses its decoded JSON body.
    pub capture_pointer: String,
    pub instance_pointer: String,
    pub openapi_pointer: String,
    pub capture_evidence_id: String,
    pub openapi_evidence_id: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaReview {
    pub exchange_id: String,
    pub operation_id: String,
    pub sources: Value,
    pub observations: Vec<SchemaObservation>,
    pub checks: usize,
    pub coverage: String,
    pub network_requests: u8,
}
fn pointer(parent: &str, key: &str) -> String {
    format!("{parent}/{}", key.replace('~', "~0").replace('/', "~1"))
}
fn source_ref(doc: &Value, at: &str) -> Result<(Value, String)> {
    let mut at = at.to_owned();
    let mut seen = BTreeSet::new();
    for _ in 0..32 {
        if !seen.insert(at.clone()) {
            return Err(AppError::new("REF_CYCLE", "Cyclic structural reference"));
        }
        let value = doc
            .pointer(&at)
            .ok_or_else(|| bad("Source pointer is missing"))?;
        match value.get("$ref").and_then(Value::as_str) {
            Some(r) if r.starts_with("#/") => at = r[1..].into(),
            Some(_) => {
                return Err(AppError::new(
                    "EXTERNAL_REF_DISABLED",
                    "External references are disabled",
                ));
            }
            None => return Ok((value.clone(), at)),
        }
    }
    Err(AppError::new(
        "REF_LIMIT",
        "Structural reference budget exceeded",
    ))
}

struct Review<'a> {
    doc: &'a Value,
    capture: &'a HttpExchange,
    report: SchemaReview,
    version30: bool,
    capture_root: String,
    active: BTreeSet<(String, String)>,
    output_bytes: usize,
    update: &'a mut dyn FnMut(u8, &str) -> Result<()>,
}
impl Review<'_> {
    fn observe(
        &mut self,
        class: &str,
        rule: &str,
        message: &str,
        capture: &str,
        instance: &str,
        schema: &str,
    ) {
        if self.report.observations.len() >= MAX_OBSERVATIONS {
            return;
        }
        let observation = SchemaObservation {
            classification: class.into(),
            rule: rule.into(),
            message: message.into(),
            capture_pointer: format!("{}{capture}", self.capture_root),
            instance_pointer: instance.into(),
            openapi_pointer: schema.into(),
            capture_evidence_id: self.report.sources["capture"]["evidence_id"]
                .as_str()
                .unwrap_or("")
                .into(),
            openapi_evidence_id: self.report.sources["openapi"]["evidence_id"]
                .as_str()
                .unwrap_or("")
                .into(),
        };
        let size = serde_json::to_vec(&observation).map_or(usize::MAX, |v| v.len());
        if self.output_bytes.saturating_add(size) > 1024 * 1024 {
            self.report.checks = MAX_CHECKS;
            if self
                .report
                .observations
                .last()
                .is_none_or(|o| o.rule != "resource_budget")
            {
                self.report.observations.push(SchemaObservation { classification:"unsupported_validation_case".into(),rule:"resource_budget".into(),message:"Observation output reached its 1 MiB budget; remaining validation was omitted".into(),capture_pointer:self.capture_root.clone(),instance_pointer:String::new(),openapi_pointer:"/paths".into(),capture_evidence_id:observation.capture_evidence_id,openapi_evidence_id:observation.openapi_evidence_id });
            }
            return;
        }
        self.output_bytes += size;
        self.report.observations.push(observation);
    }
    fn step(&mut self, capture: &str, instance: &str, schema: &str) -> Result<bool> {
        if self.report.checks >= MAX_CHECKS
            || self.report.observations.len() >= MAX_OBSERVATIONS - 1
        {
            if self
                .report
                .observations
                .last()
                .is_none_or(|o| o.rule != "resource_budget")
            {
                self.observe("unsupported_validation_case", "resource_budget", "Validation stopped at its check or observation budget; remaining content was not evaluated", capture, instance, schema);
            }
            return Ok(false);
        }
        self.report.checks += 1;
        if self.report.checks.is_multiple_of(64) {
            (self.update)(60, "Checking bounded offline schema constraints")?;
        }
        Ok(true)
    }
    fn schema(
        &mut self,
        value: &Value,
        at: &str,
        body: &str,
        instance: &str,
        response: bool,
        depth: usize,
    ) -> Result<()> {
        if !self.step(body, instance, at)? {
            return Ok(());
        }
        if depth > 32 {
            self.observe(
                "unsupported_validation_case",
                "schema_depth",
                "Schema recursion exceeds 32 levels",
                body,
                instance,
                at,
            );
            return Ok(());
        }
        if value
            .as_str()
            .is_some_and(|s| s.contains("[REDACTED]") || s.starts_with("[Body withheld:"))
        {
            self.observe(
                "insufficient_evidence",
                "redacted_value",
                "Value was redacted or withheld; its type and constraints were not assessed",
                body,
                instance,
                at,
            );
            return Ok(());
        }
        let Some(schema) = self.doc.pointer(at) else {
            self.observe(
                "unsupported_validation_case",
                "schema_reference",
                "Schema pointer cannot be resolved",
                body,
                instance,
                at,
            );
            return Ok(());
        };
        if schema.get("$id").is_some() || schema.get("$schema").is_some_and(|v| !known_dialect(v)) {
            self.observe(
                "unsupported_validation_case",
                "schema_dialect",
                "Custom schema dialect or resource identifier is unsupported",
                body,
                instance,
                at,
            );
            return Ok(());
        }
        if let Some(r) = schema.get("$ref").and_then(Value::as_str) {
            if !r.starts_with("#/") || !self.active.insert((r.to_owned(), instance.to_owned())) {
                self.observe(
                    "unsupported_validation_case",
                    "schema_reference_cycle",
                    "External or non-progressing cyclic schema reference was not expanded",
                    body,
                    instance,
                    at,
                );
                return Ok(());
            }
            self.schema(value, &r[1..], body, instance, response, depth + 1)?;
            self.active.remove(&(r.to_owned(), instance.to_owned()));
            if self.version30 {
                return Ok(());
            }
            // 3.1 $ref siblings still apply. The annotation below documents partial coverage.
        }
        if schema.is_boolean() {
            if self.version30 {
                self.observe(
                    "unsupported_validation_case",
                    "boolean_schema",
                    "Boolean schemas are not supported in OpenAPI 3.0",
                    body,
                    instance,
                    at,
                );
            } else if schema == false {
                self.observe(
                    "confirmed_documented_mismatch",
                    "false_schema",
                    "Captured value is disallowed by a false schema",
                    body,
                    instance,
                    at,
                );
            }
            return Ok(());
        }
        let Some(object) = schema.as_object() else {
            self.observe(
                "unsupported_validation_case",
                "invalid_schema",
                "Schema is not an object or supported boolean",
                body,
                instance,
                at,
            );
            return Ok(());
        };
        // URI scope/dialect changes require a full resource resolver; never guess their meaning.
        if object.contains_key("$id") || object.get("$schema").is_some_and(|v| !known_dialect(v)) {
            self.observe(
                "unsupported_validation_case",
                "schema_dialect",
                "Custom schema dialect or resource identifier is unsupported",
                body,
                instance,
                at,
            );
            return Ok(());
        }
        const SUPPORTED: &[&str] = &[
            "$ref",
            "$schema",
            "$defs",
            "definitions",
            "type",
            "nullable",
            "required",
            "properties",
            "items",
            "readOnly",
            "writeOnly",
            "title",
            "description",
            "example",
            "examples",
            "default",
            "deprecated",
            "xml",
            "externalDocs",
        ];
        if object
            .keys()
            .any(|k| !SUPPORTED.contains(&k.as_str()) && !k.starts_with("x-"))
        {
            self.observe("unsupported_validation_case", "schema_keywords", "Additional schema constraints are present and were not evaluated; supported checks below are partial", body, instance, at);
        }
        if let Some(types) = object.get("type") {
            let types: Vec<_> = if let Some(t) = types.as_str() {
                vec![t]
            } else if !self.version30 {
                types
                    .as_array()
                    .filter(|a| a.iter().all(Value::is_string))
                    .map(|a| a.iter().filter_map(Value::as_str).collect())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            if types.is_empty()
                || (self.version30 && types.contains(&"null"))
                || types.iter().any(|t| {
                    ![
                        "null", "object", "array", "string", "boolean", "number", "integer",
                    ]
                    .contains(t)
                })
            {
                self.observe(
                    "unsupported_validation_case",
                    "schema_type",
                    "Type declaration is unsupported or invalid",
                    body,
                    instance,
                    &pointer(at, "type"),
                );
            } else {
                let nullable = self.version30 && object.get("nullable") == Some(&Value::Bool(true));
                let matches = (nullable && value.is_null())
                    || types.iter().any(|t| match *t {
                        "null" => value.is_null(),
                        "object" => value.is_object(),
                        "array" => value.is_array(),
                        "string" => value.is_string(),
                        "boolean" => value.is_boolean(),
                        "number" => value.is_number(),
                        "integer" => value.as_f64().is_some_and(|n| n.fract() == 0.0),
                        _ => false,
                    });
                if !matches {
                    self.observe(
                        "confirmed_documented_mismatch",
                        "json_type",
                        "Captured JSON type differs from the documented type",
                        body,
                        instance,
                        &pointer(at, "type"),
                    );
                    return Ok(());
                }
            }
        }
        if let Some(data) = value.as_object() {
            if let Some(required) = object.get("required") {
                if let Some(required) = required
                    .as_array()
                    .filter(|a| a.iter().all(Value::is_string))
                {
                    for name in required.iter().filter_map(Value::as_str) {
                        if !self.step(body, instance, at)? {
                            break;
                        }
                        if data.contains_key(name) {
                            continue;
                        }
                        let prop = pointer(&pointer(at, "properties"), name);
                        // Direction annotations can be behind $ref; failure is inconclusive.
                        let annotation = self
                            .doc
                            .pointer(&prop)
                            .map(|_| source_ref(self.doc, &prop))
                            .transpose();
                        let annotated = annotation
                            .as_ref()
                            .ok()
                            .and_then(Option::as_ref)
                            .map(|p| &p.0);
                        let direct = self.doc.pointer(&prop);
                        let directional = [annotated, direct].into_iter().flatten().any(|p| {
                            p.get(if response { "writeOnly" } else { "readOnly" })
                                == Some(&Value::Bool(true))
                        });
                        if directional || annotation.is_err() {
                            if !self.version30 || annotation.is_err() {
                                self.observe("insufficient_evidence", "directional_required", "Required field has directional annotations or an unresolved reference; application policy must decide", body, instance, &pointer(at,"required"));
                            }
                            continue;
                        }
                        self.observe(
                            "confirmed_documented_mismatch",
                            "required_field",
                            "Captured JSON object lacks a required property",
                            body,
                            &pointer(instance, name),
                            &pointer(at, "required"),
                        );
                    }
                } else {
                    self.observe(
                        "unsupported_validation_case",
                        "invalid_required",
                        "Required-property declaration is invalid",
                        body,
                        instance,
                        &pointer(at, "required"),
                    );
                }
            }
            if let Some(properties) = object.get("properties").and_then(Value::as_object) {
                for (name, child) in data {
                    if properties.contains_key(name) {
                        self.schema(
                            child,
                            &pointer(&pointer(at, "properties"), name),
                            body,
                            &pointer(instance, name),
                            response,
                            depth + 1,
                        )?;
                    }
                }
            }
        }
        if let Some(data) = value.as_array() {
            if object
                .get("items")
                .is_some_and(|v| v.is_object() || v.is_boolean())
            {
                for (i, child) in data.iter().enumerate() {
                    self.schema(
                        child,
                        &pointer(at, "items"),
                        body,
                        &pointer(instance, &i.to_string()),
                        response,
                        depth + 1,
                    )?;
                    if self.report.checks >= MAX_CHECKS {
                        break;
                    }
                }
            } else if object.contains_key("items") {
                self.observe(
                    "unsupported_validation_case",
                    "array_items",
                    "Tuple or invalid items schema is unsupported",
                    body,
                    instance,
                    &pointer(at, "items"),
                );
            }
        }
        Ok(())
    }
    fn content(&mut self, declaration: &Value, at: &str, response: bool) -> Result<()> {
        let side = if response { "response" } else { "request" };
        let message = if response {
            &self.capture.response
        } else {
            &self.capture.request
        };
        let body = message["body"].clone();
        let body_at = if response {
            "/response/content/text"
        } else {
            "/request/postData/text"
        };
        let content_at = pointer(at, "content");
        let Some(content) = declaration.get("content").and_then(Value::as_object) else {
            return Ok(());
        };
        let header = header_value(message, "content-type");
        if header.as_ref().is_some_and(|h| {
            !h.contains("[REDACTED]")
                && !body["mime_type"].as_str().unwrap_or("").is_empty()
                && !h
                    .split(';')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .eq_ignore_ascii_case(
                        body["mime_type"]
                            .as_str()
                            .unwrap_or("")
                            .split(';')
                            .next()
                            .unwrap_or("")
                            .trim(),
                    )
        }) {
            self.observe("possible_mismatch","capture_content_type_conflict","HAR body MIME metadata differs from its Content-Type header; the header selects documentation", &format!("/{side}/headers"),"",&content_at);
        }
        let mime = header
            .as_deref()
            .filter(|m| !m.contains("[REDACTED]"))
            .unwrap_or_else(|| body["mime_type"].as_str().unwrap_or(""));
        if mime.is_empty() {
            self.observe(
                "insufficient_evidence",
                "content_type",
                "Capture has no usable Content-Type; body schema was not selected",
                &format!("/{side}/headers"),
                "",
                &content_at,
            );
            return Ok(());
        }
        let mime = mime
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        let media = content
            .keys()
            .filter(|key| media_matches(key, &mime))
            .max_by_key(|key| key.bytes().filter(|b| *b != b'*').count());
        let Some(media) = media else {
            self.observe(
                "confirmed_documented_mismatch",
                "content_type",
                "Captured media type is not among documented content types",
                &format!("/{side}/headers"),
                "",
                &content_at,
            );
            return Ok(());
        };
        let schema_at = pointer(&pointer(&content_at, media), "schema");
        if self.doc.pointer(&schema_at).is_none() {
            return Ok(());
        }
        if body["decoded_size"].as_u64() == Some(0) {
            self.observe(
                "insufficient_evidence",
                "body_unavailable",
                "HAR body is empty or omitted; no missing-field or type verdict was made",
                body_at,
                "",
                &schema_at,
            );
        } else if mime == "application/json" || mime.ends_with("+json") {
            self.schema(&body["preview"], &schema_at, body_at, "", response, 0)?;
        } else {
            self.observe(
                "unsupported_validation_case",
                "body_encoding",
                "Only retained JSON body structure is validated",
                body_at,
                "",
                &schema_at,
            );
        }
        Ok(())
    }
}
fn known_dialect(value: &Value) -> bool {
    value.as_str().is_some_and(|s| {
        [
            "https://spec.openapis.org/oas/3.1/dialect/base",
            "https://json-schema.org/draft/2020-12/schema",
        ]
        .contains(&s.trim_end_matches('#'))
    })
}
fn header_value(message: &Value, name: &str) -> Option<String> {
    message["headers"].as_array()?.iter().find(|h| {
        h["name"]
            .as_str()
            .is_some_and(|n| n.eq_ignore_ascii_case(name))
    })?["value"]
        .as_str()
        .map(str::to_owned)
}
fn media_matches(pattern: &str, actual: &str) -> bool {
    let p = pattern.to_ascii_lowercase();
    p == actual
        || p == "*/*"
        || p.strip_suffix("/*")
            .is_some_and(|t| actual.starts_with(&format!("{t}/")))
}

impl Core {
    fn schema_sources(
        &self,
        capture: &HttpExchange,
        operation: &ApiOperation,
    ) -> Result<(Value, Value)> {
        self.require_http_review_source(capture)?;
        if capture.project_id != operation.project_id || capture.target_id != operation.target_id {
            return Err(AppError::new(
                "TARGET_MISMATCH",
                "Schema review requires the same project and exact target partition",
            ));
        }
        let url = Url::parse(&capture.url).map_err(|_| bad("Invalid retained URL"))?;
        let patterns = operation_patterns(&operation.servers, &operation.path)?;
        if capture.method != operation.method
            || !patterns.iter().any(|p| {
                p.origin.is_some()
                    && matches_pattern(
                        &url.origin().ascii_serialization(),
                        &url.path().split('/').collect::<Vec<_>>(),
                        p,
                    )
            })
        {
            return Err(AppError::new(
                "OPERATION_MISMATCH",
                "Select an operation matching the exact scheme, host, port, method and path; absolute servers are required",
            ));
        }
        let source: (String,String) = self.db.connect()?.query_row("SELECT evidence_id,sha256 FROM http_imports WHERE id=?1 AND kind='openapi' AND project_id=?2",params![operation.import_id,operation.project_id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(db_error)?;
        self.evidence_verify(&source.0)?;
        let bytes = super::super::evidence::read_regular(
            &self.output_path("evidence", &source.0)?,
            MAX_HTTP_IMPORT,
        )?;
        if super::super::evidence::digest(&bytes) != source.1 {
            return Err(AppError::new(
                "INTEGRITY_FAILED",
                "OpenAPI source hash differs from its import record",
            ));
        }
        let doc = openapi_document(&bytes)?;
        // Integrity of an attachment alone does not prove normalized SQLite metadata agrees.
        let input = HttpImportInput {
            project_id: operation.project_id.clone(),
            target_id: operation.target_id.clone(),
            title: String::new(),
            identity: String::new(),
            role: String::new(),
        };
        let (operations, _) = parse_openapi(&bytes, &input, &operation.import_id)?;
        let mut original = operations
            .into_iter()
            .find(|o| o.method == operation.method && o.path == operation.path)
            .ok_or_else(|| {
                AppError::new(
                    "SOURCE_CHANGED",
                    "Operation is absent from its original OpenAPI document",
                )
            })?;
        original.id = operation.id.clone();
        if serde_json::to_value(&original).map_err(json_error)?
            != serde_json::to_value(operation).map_err(json_error)?
        {
            return Err(AppError::new(
                "SOURCE_CHANGED",
                "Normalized operation differs from its original document",
            ));
        }
        let sources = json!({"capture":self.http_source_integrity(capture)?,"openapi":{"import_id":operation.import_id,"evidence_id":source.0,"sha256":source.1}});
        Ok((doc, sources))
    }
    /// The digest binds normalized capture/operation records and both original hashes.
    fn schema_fingerprint(
        &self,
        capture: &HttpExchange,
        operation: &ApiOperation,
        sources: &Value,
    ) -> Result<String> {
        Ok(super::super::evidence::digest(
            &serde_json::to_vec(&json!([capture, operation, sources])).map_err(json_error)?,
        ))
    }
    pub fn http_schema_enqueue(&self, exchange: &str, operation: &str) -> Result<AnalysisJob> {
        let capture = self.http_exchange(exchange)?;
        let operation = self.api_operation(operation)?;
        let (_, sources) = self.schema_sources(&capture, &operation)?;
        let fingerprint = self.schema_fingerprint(&capture, &operation, &sources)?;
        let job = Uuid::new_v4().to_string();
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let active:i64 = tx.query_row("SELECT count(*) FROM http_schema_jobs r JOIN analysis_jobs j ON j.id=r.job_id WHERE r.exchange_id=?1 AND r.operation_id=?2 AND j.status IN ('queued','running')",params![exchange,operation.id],|r|r.get(0)).map_err(db_error)?;
        if active != 0 {
            return Err(AppError::new(
                "JOB_BUSY",
                "This capture/operation pair already has a schema job",
            ));
        }
        tx.execute("INSERT INTO analysis_jobs(id,project_id,target_id,kind,status,created_at,timeout_ms,logs_json) VALUES(?1,?2,?3,'offline_schema_review','queued',?4,30000,'[\"Offline schema observation requested; no network requests\"]')",params![job,capture.project_id,capture.target_id,now()]).map_err(db_error)?;
        tx.execute(
            "INSERT INTO http_schema_jobs VALUES(?1,?2,?3,?4)",
            params![job, exchange, operation.id, fingerprint],
        )
        .map_err(db_error)?;
        Self::audit(&tx, "http.schema_queued", &job, "offline")?;
        tx.commit().map_err(db_error)?;
        self.job(&job)
    }
    pub(crate) fn prepare_http_schema(
        &self,
        job: &AnalysisJob,
        update: &mut dyn FnMut(u8, &str) -> Result<()>,
    ) -> Result<super::super::jobs::PreparedJob> {
        let (capture_id, operation_id, fingerprint): (String, String, String) = self
            .db
            .connect()?
            .query_row(
                "SELECT exchange_id,operation_id,fingerprint FROM http_schema_jobs WHERE job_id=?1",
                [&job.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(db_error)?;
        let capture = self.http_exchange(&capture_id)?;
        let operation = self.api_operation(&operation_id)?;
        let (doc, sources) = self.schema_sources(&capture, &operation)?;
        if capture.project_id != job.project_id
            || capture.target_id != job.target_id
            || self.schema_fingerprint(&capture, &operation, &sources)? != fingerprint
        {
            return Err(AppError::new(
                "SOURCE_CHANGED",
                "Schema inputs changed after the job was queued",
            ));
        }
        update(
            25,
            "Both source hashes and exact target/origin binding checked",
        )?;
        let capture_root = self.schema_capture_pointer(&capture)?;
        let report = SchemaReview { exchange_id:capture.id.clone(), operation_id:operation.id.clone(), sources, observations:Vec::new(), checks:0, coverage:"partial: type/required/properties/items and HTTP metadata only; no vulnerability verdict".into(), network_requests:0 };
        let mut review = Review {
            doc: &doc,
            capture: &capture,
            report,
            version30: doc["openapi"]
                .as_str()
                .is_some_and(|v| v.starts_with("3.0.")),
            capture_root,
            active: BTreeSet::new(),
            output_bytes: 0,
            update,
        };
        review.operation(&operation)?;
        (review.update)(
            85,
            "Offline observations completed; unsupported cases remain explicit",
        )?;
        self.schema_sources(&capture, &operation)?;
        Ok(super::super::jobs::PreparedJob {
            result: safe_value(serde_json::to_value(review.report).map_err(json_error)?),
            analysis: None,
            http: None,
        })
    }
    fn schema_capture_pointer(&self, capture: &HttpExchange) -> Result<String> {
        // Preserve pre-011 imports: match immutable normalized fields, refusing ambiguity.
        let source = self.http_source_integrity(capture)?;
        let bytes = super::super::evidence::read_regular(
            &self.output_path("evidence", source["evidence_id"].as_str().unwrap_or(""))?,
            MAX_HTTP_IMPORT,
        )?;
        if super::super::evidence::digest(&bytes) != source["sha256"].as_str().unwrap_or("") {
            return Err(AppError::new(
                "INTEGRITY_FAILED",
                "HAR changed during source validation",
            ));
        }
        let input = HttpImportInput {
            project_id: capture.project_id.clone(),
            target_id: capture.target_id.clone(),
            title: String::new(),
            identity: capture.identity.clone(),
            role: capture.role.clone(),
        };
        let (entries, _) = parse_har(&bytes, &input, &capture.import_id)?;
        let matches: Vec<_> = entries
            .into_iter()
            .filter(|e| {
                let mut expected = capture.clone();
                expected.id = e.id.clone();
                expected.source_pointer = e.source_pointer.clone();
                capture
                    .source_pointer
                    .as_ref()
                    .is_none_or(|at| Some(at) == e.source_pointer.as_ref())
                    && serde_json::to_value(&expected).ok() == serde_json::to_value(e).ok()
            })
            .collect();
        if matches.len() != 1 {
            return Err(AppError::new(
                "SOURCE_AMBIGUOUS",
                "Older capture cannot be uniquely located in its original HAR; reimport it for exact provenance",
            ));
        }
        matches[0]
            .source_pointer
            .clone()
            .ok_or_else(|| bad("Capture source pointer is missing"))
    }
}

impl Review<'_> {
    fn operation(&mut self, operation: &ApiOperation) -> Result<()> {
        let item_at = pointer("/paths", &operation.path);
        let (_, item_at) = source_ref(self.doc, &item_at)?;
        let (op, at) = source_ref(
            self.doc,
            &pointer(&item_at, &operation.method.to_ascii_lowercase()),
        )?;
        if self
            .doc
            .get("jsonSchemaDialect")
            .is_some_and(|d| !known_dialect(d))
        {
            self.observe(
                "unsupported_validation_case",
                "schema_dialect",
                "Document selects an unsupported JSON Schema dialect",
                "/response/content/text",
                "",
                "/jsonSchemaDialect",
            );
            return Ok(());
        }
        let responses = &op["responses"];
        let exact = self.capture.status.to_string();
        let range = format!("{}XX", self.capture.status / 100);
        let key = [exact.as_str(), range.as_str(), "default"]
            .into_iter()
            .find(|k| responses.get(*k).is_some());
        if let Some(key) = key {
            let (response, response_at) =
                source_ref(self.doc, &pointer(&pointer(&at, "responses"), key))?;
            self.required_headers(&response, &response_at)?;
            self.content(&response, &response_at, true)?;
        } else {
            self.observe(
                if self.capture.status == 0 {
                    "insufficient_evidence"
                } else {
                    "confirmed_documented_mismatch"
                },
                "response_status",
                "Captured status has no explicit, range or default response definition",
                "/response/status",
                "",
                &pointer(&at, "responses"),
            );
        }
        if op.get("requestBody").is_some() {
            let (request, request_at) = source_ref(self.doc, &pointer(&at, "requestBody"))?;
            if request.get("required") == Some(&Value::Bool(true))
                && self.capture.request["body"]["decoded_size"] == 0
            {
                self.observe("possible_mismatch","request_body_required","Request body is documented as required but HAR has no retained bytes; capture may be incomplete","/request/postData","",&pointer(&request_at,"required"));
            }
            self.content(&request, &request_at, false)?;
        }
        self.parameters(&item_at, &at)?;
        self.authentication(&op, &at)?;
        Ok(())
    }
    fn required_headers(&mut self, response: &Value, at: &str) -> Result<()> {
        if let Some(headers) = response.get("headers").and_then(Value::as_object) {
            for name in headers.keys() {
                let (header, header_at) =
                    source_ref(self.doc, &pointer(&pointer(at, "headers"), name))?;
                if name.eq_ignore_ascii_case("content-type") {
                    continue;
                } // OAS ignores this response-header declaration.
                if header.get("required") == Some(&Value::Bool(true))
                    && header_value(&self.capture.response, name).is_none()
                {
                    self.observe(
                        "possible_mismatch",
                        "required_response_header",
                        "A documented required response header is absent from this capture",
                        "/response/headers",
                        "",
                        &header_at,
                    );
                }
            }
        }
        Ok(())
    }
    fn parameters(&mut self, item_at: &str, op_at: &str) -> Result<()> {
        let mut parameters = BTreeMap::new();
        for parent in [item_at, op_at] {
            let at = pointer(parent, "parameters");
            if let Some(items) = self.doc.pointer(&at).and_then(Value::as_array) {
                for i in 0..items.len() {
                    let (p, p_at) = source_ref(self.doc, &pointer(&at, &i.to_string()))?;
                    parameters.insert(
                        (string(&p, "in")?.to_owned(), string(&p, "name")?.to_owned()),
                        (p, p_at),
                    );
                }
            }
        }
        for ((location, name), (parameter, at)) in parameters {
            if !self.step("/request", "", &at)? {
                break;
            }
            let present = match location.as_str() {
                "query" => self.capture.query_parameters.contains(&name),
                "header" => header_value(&self.capture.request, &name).is_some(),
                "cookie" => self.capture.request["cookies"]
                    .as_array()
                    .is_some_and(|cs| cs.iter().any(|c| c["name"] == name)),
                "path" => true, // Exact matched template establishes segment presence, never its serialized type.
                _ => false,
            };
            if parameter.get("required") == Some(&Value::Bool(true)) && !present {
                self.observe(
                    "possible_mismatch",
                    "required_parameter",
                    "A required parameter is absent from captured request metadata",
                    "/request",
                    "",
                    &at,
                );
            } else if present && parameter.get("schema").is_some() {
                self.observe("insufficient_evidence","parameter_value","Parameter presence observed; values are redacted or serialized and were not type-validated","/request","",&pointer(&at,"schema"));
            }
        }
        Ok(())
    }
    fn authentication(&mut self, op: &Value, at: &str) -> Result<()> {
        let at = if op.get("security").is_some() {
            pointer(at, "security")
        } else {
            "/security".into()
        };
        let Some(requirements) = self.doc.pointer(&at).and_then(Value::as_array) else {
            return Ok(());
        };
        if requirements.is_empty()
            || requirements
                .iter()
                .any(|r| r.as_object().is_some_and(|o| o.is_empty()))
        {
            return Ok(());
        }
        // OR across requirements, AND inside each requirement. Only mechanism presence is observable.
        let mut possible = false;
        let mut unknown = false;
        for requirement in requirements {
            let Some(schemes) = requirement.as_object() else {
                unknown = true;
                continue;
            };
            let mut all = true;
            for name in schemes.keys() {
                let path = pointer("/components/securitySchemes", name);
                let (scheme, _) = source_ref(self.doc, &path)?;
                let has = match scheme["type"].as_str() {
                    Some("http")
                        if scheme["scheme"]
                            .as_str()
                            .is_some_and(|s| s.eq_ignore_ascii_case("bearer")) =>
                    {
                        self.capture.authentication.iter().any(|a| a == "Bearer")
                    }
                    Some("http")
                        if scheme["scheme"]
                            .as_str()
                            .is_some_and(|s| s.eq_ignore_ascii_case("basic")) =>
                    {
                        self.capture.authentication.iter().any(|a| a == "Basic")
                    }
                    Some("apiKey") => {
                        let n = scheme["name"].as_str().unwrap_or("");
                        match scheme["in"].as_str() {
                            Some("header") => header_value(&self.capture.request, n).is_some(),
                            Some("query") => self.capture.query_parameters.iter().any(|v| v == n),
                            Some("cookie") => self.capture.request["cookies"]
                                .as_array()
                                .is_some_and(|cs| cs.iter().any(|c| c["name"] == n)),
                            _ => {
                                unknown = true;
                                false
                            }
                        }
                    }
                    _ => {
                        unknown = true;
                        false
                    }
                };
                all &= has;
            }
            possible |= all;
        }
        self.observe(if unknown || possible {"insufficient_evidence"} else {"possible_mismatch"},"authentication_scheme",if possible {"Documented authentication mechanism appears in capture; credential validity and authorization cannot be inferred"} else if unknown {"Authentication scheme requires unsupported contextual validation"} else {"No complete documented authentication alternative appears in captured metadata; capture may be incomplete"},"/request/headers","",&at);
        Ok(())
    }
}
