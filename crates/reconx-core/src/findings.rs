use super::*;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    #[default]
    Hypothesis,
    NeedsVerification,
    Verified,
    Refuted,
    FalsePositive,
    Duplicate,
    AcceptedRisk,
    Remediated,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    #[default]
    Informational,
    Low,
    Medium,
    High,
    Critical,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FindingInput {
    pub project_id: String,
    pub target_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub affected_asset: String,
    #[serde(default)]
    pub category: String,
    pub cwe: Option<String>,
    pub cve: Option<String>,
    pub cvss_v3: Option<String>,
    pub cvss_v4: Option<String>,
    #[serde(default)]
    pub severity: Severity,
    #[serde(default)]
    pub confidence: Confidence,
    #[serde(default)]
    pub status: FindingStatus,
    #[serde(default)]
    pub technical_details: String,
    #[serde(default)]
    pub reproduction_steps: String,
    #[serde(default)]
    pub impact: String,
    #[serde(default)]
    pub remediation: String,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    #[serde(default)]
    pub verification_note: String,
    #[serde(default)]
    pub ai_generated: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    #[default]
    Low,
    Medium,
    High,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub input: FindingInput,
    pub revision: u32,
    pub created_at: String,
    pub updated_at: String,
    pub duplicate_of: Option<String>,
    pub cvss: Vec<CvssScore>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvssScore {
    pub version: String,
    pub vector: String,
    pub score: f64,
    pub scope: String,
}
pub fn calculate_cvss(vector: &str) -> Result<CvssScore> {
    if vector.len() > 1024 {
        return Err(AppError::new("INVALID_CVSS", "CVSS vector is too long"));
    }
    // The library accepts partial vectors using defaults. Product scoring must
    // require every base metric, never invent missing metrics or accept duplicates.
    let required: &[&str] = if vector.starts_with("CVSS:3.1/") {
        &["AV", "AC", "PR", "UI", "S", "C", "I", "A"]
    } else if vector.starts_with("CVSS:4.0/") {
        &[
            "AV", "AC", "AT", "PR", "UI", "VC", "VI", "VA", "SC", "SI", "SA",
        ]
    } else {
        return Err(AppError::new(
            "INVALID_CVSS",
            "Supported vector prefixes: CVSS:3.1/ and CVSS:4.0/",
        ));
    };
    let mut metrics = std::collections::BTreeSet::new();
    for part in vector.split('/').skip(1) {
        let (key, value) = part.split_once(':').ok_or_else(|| {
            AppError::new(
                "INVALID_CVSS",
                "Each supplied metric needs a name and value",
            )
        })?;
        if value.is_empty() || !metrics.insert(key) || (required.contains(&key) && value == "X") {
            return Err(AppError::new(
                "INVALID_CVSS",
                "Duplicate, empty or undefined required metrics are not allowed",
            ));
        }
    }
    if required.iter().any(|key| !metrics.contains(key)) {
        return Err(AppError::new(
            "INVALID_CVSS",
            &format!(
                "Complete all required base metrics: {}",
                required.join(", ")
            ),
        ));
    }
    let (version, score, scope) = if vector.starts_with("CVSS:3.1/") {
        let v:cvss::v3::Vector=vector.parse().map_err(|_|AppError::new("INVALID_CVSS","Provide a complete CVSS 3.1 vector; required base metrics: AV, AC, PR, UI, S, C, I, A"))?;
        ("3.1", v.base_score().value(), "base")
    } else if vector.starts_with("CVSS:4.0/") {
        let v:cvss::v4::Vector=vector.parse().map_err(|_|AppError::new("INVALID_CVSS","Provide a complete CVSS 4.0 vector; required metrics: AV, AC, AT, PR, UI, VC, VI, VA, SC, SI, SA"))?;
        ("4.0", v.score().value(), "supplied metric groups")
    } else {
        return Err(AppError::new(
            "INVALID_CVSS",
            "Supported vector prefixes: CVSS:3.1/ and CVSS:4.0/",
        ));
    };
    Ok(CvssScore {
        version: version.into(),
        vector: vector.into(),
        score,
        scope: scope.into(),
    })
}
type StoredFinding = (String, u32, String, String, Option<String>, Option<String>);
impl Core {
    fn validate_finding(&self, input: &mut FindingInput, confirm: bool) -> Result<Vec<CvssScore>> {
        let project = self.project(&input.project_id)?;
        input.project_id = project.id;
        if project.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore project before editing findings",
            ));
        }
        input.title = validate_text(&input.title, 200, "Finding title")?;
        if serde_json::to_vec(input).map_err(json_error)?.len() > 128 * 1024 {
            return Err(AppError::new("INPUT_LIMIT", "Finding exceeds 128 KiB"));
        }
        if let Some(target) = &input.target_id
            && self.target(target)?.input.project_id != input.project_id
        {
            return Err(AppError::new(
                "PROJECT_MISMATCH",
                "Target belongs to another project",
            ));
        }
        for (value, pattern, label) in [
            (&input.cwe, r"^CWE-[1-9][0-9]{0,5}$", "CWE"),
            (&input.cve, r"^CVE-[0-9]{4}-[0-9]{4,10}$", "CVE"),
        ] {
            if value
                .as_ref()
                .is_some_and(|v| !regex::Regex::new(pattern).unwrap().is_match(v))
            {
                return Err(AppError::new(
                    "INVALID_INPUT",
                    &format!("Invalid {label} identifier; leave unknown identifiers empty"),
                ));
            }
        }
        let mut scores = Vec::new();
        for (value, prefix) in [(&input.cvss_v3, "CVSS:3.1/"), (&input.cvss_v4, "CVSS:4.0/")] {
            if let Some(value) = value {
                if !value.starts_with(prefix) {
                    return Err(AppError::new(
                        "INVALID_CVSS",
                        "CVSS vector version does not match its field",
                    ));
                }
                scores.push(calculate_cvss(value)?);
            }
        }
        if input.references.len() > 50 || input.evidence_ids.len() > 100 {
            return Err(AppError::new(
                "INPUT_LIMIT",
                "At most 50 references and 100 evidence links per finding",
            ));
        }
        input.evidence_ids.sort();
        input.evidence_ids.dedup();
        let mut supporting = false;
        for id in &input.evidence_ids {
            let e = self.evidence(id)?;
            if e.project_id != input.project_id
                || (e.target_id.is_some()
                    && input.target_id.is_some()
                    && e.target_id != input.target_id)
            {
                return Err(AppError::new(
                    "PROJECT_MISMATCH",
                    "Evidence must belong to the same project and applicable target",
                ));
            }
            self.evidence_bytes(&e)?;
            supporting |=
                e.kind != EvidenceKind::AiSummary && e.verification_state == "manually_reviewed";
        }
        if input.status == FindingStatus::Verified
            && (!confirm || !supporting || input.verification_note.trim().len() < 8)
        {
            return Err(AppError::new(
                "VERIFICATION_REQUIRED",
                "Verified requires explicit manual confirmation, a review note and at least one integrity-checked, manually reviewed non-AI evidence record",
            ));
        }
        Ok(scores)
    }
    pub fn finding_save(
        &self,
        id: Option<&str>,
        mut input: FindingInput,
        revision: Option<u32>,
        confirm_verified: bool,
    ) -> Result<Finding> {
        let scores = self.validate_finding(&mut input, confirm_verified)?;
        let old = id.map(|id| self.finding(id)).transpose()?;
        if old
            .as_ref()
            .is_some_and(|f| f.input.project_id != input.project_id)
        {
            return Err(AppError::new(
                "PROJECT_MISMATCH",
                "A finding cannot move to another project",
            ));
        }
        if old.as_ref().is_some_and(|f| revision != Some(f.revision)) {
            return Err(AppError::new(
                "STALE_RECORD",
                "Finding changed; reload before saving",
            ));
        }
        let f = Finding {
            id: old
                .as_ref()
                .map(|f| f.id.clone())
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            created_at: old
                .as_ref()
                .map(|f| f.created_at.clone())
                .unwrap_or_else(now),
            updated_at: now(),
            revision: old.as_ref().map_or(1, |f| f.revision + 1),
            duplicate_of: old.as_ref().and_then(|f| f.duplicate_of.clone()),
            input,
            cvss: scores,
        };
        let status = serde_json::to_value(f.input.status).map_err(json_error)?;
        let severity = serde_json::to_value(f.input.severity).map_err(json_error)?;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let data = serde_json::to_string(&f.input).map_err(json_error)?;
        if old.is_some() {
            let n=tx.execute("UPDATE findings SET target_id=?1,title=?2,status=?3,severity=?4,revision=?5,updated_at=?6,data_json=?7 WHERE id=?8 AND revision=?9",params![f.input.target_id,f.input.title,status.as_str(),severity.as_str(),f.revision,f.updated_at,data,f.id,revision]).map_err(db_error)?;
            if n != 1 {
                return Err(AppError::new(
                    "STALE_RECORD",
                    "Finding changed during save; reload before editing",
                ));
            }
            tx.execute("DELETE FROM finding_evidence WHERE finding_id=?1", [&f.id])
                .map_err(db_error)?;
        } else {
            tx.execute("INSERT INTO findings(id,project_id,target_id,title,status,severity,revision,created_at,updated_at,data_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![f.id,f.input.project_id,f.input.target_id,f.input.title,status.as_str(),severity.as_str(),f.revision,f.created_at,f.updated_at,data]).map_err(db_error)?;
        }
        for e in &f.input.evidence_ids {
            tx.execute(
                "INSERT INTO finding_evidence(finding_id,evidence_id) VALUES(?1,?2)",
                params![f.id, e],
            )
            .map_err(db_error)?;
        }
        Self::audit(
            &tx,
            "finding.saved",
            &f.id,
            if confirm_verified {
                "manually_reviewed"
            } else {
                "completed"
            },
        )?;
        tx.commit().map_err(db_error)?;
        Ok(f)
    }
    pub fn finding(&self, id: &str) -> Result<Finding> {
        let conn = self.db.connect()?;
        let row:Option<StoredFinding>=conn.query_row("SELECT data_json,revision,created_at,updated_at,duplicate_of,target_id FROM findings WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional().map_err(db_error)?;
        let (data, revision, created_at, updated_at, duplicate_of, target) =
            row.ok_or_else(|| AppError::new("NOT_FOUND", "Finding does not exist"))?;
        let mut input: FindingInput = serde_json::from_str(&data).map_err(json_error)?;
        input.target_id = target;
        let mut stmt = conn
            .prepare(
                "SELECT evidence_id FROM finding_evidence WHERE finding_id=?1 ORDER BY evidence_id",
            )
            .map_err(db_error)?;
        input.evidence_ids = stmt
            .query_map([id], |r| r.get(0))
            .map_err(db_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(db_error)?;
        let cvss = [input.cvss_v3.as_deref(), input.cvss_v4.as_deref()]
            .into_iter()
            .flatten()
            .map(calculate_cvss)
            .collect::<Result<Vec<_>>>()?;
        Ok(Finding {
            id: id.into(),
            input,
            revision,
            created_at,
            updated_at,
            duplicate_of,
            cvss,
        })
    }
    pub fn findings(&self, project: Option<&str>) -> Result<Vec<Finding>> {
        let project = project
            .map(|id| self.project(id).map(|p| p.id))
            .transpose()?;
        let conn = self.db.connect()?;
        let mut stmt=conn.prepare("SELECT id FROM findings WHERE ?1 IS NULL OR project_id=?1 ORDER BY updated_at DESC,id LIMIT 1000").map_err(db_error)?;
        let ids = stmt
            .query_map([project], |r| r.get::<_, String>(0))
            .map_err(db_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(db_error)?;
        ids.into_iter().map(|id| self.finding(&id)).collect()
    }
    pub fn finding_remove(&self, id: &str, revision: u32) -> Result<()> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if tx
            .execute(
                "DELETE FROM findings WHERE id=?1 AND revision=?2",
                params![id, revision],
            )
            .map_err(db_error)?
            != 1
        {
            return Err(AppError::new(
                "STALE_RECORD",
                "Finding changed or does not exist",
            ));
        }
        Self::audit(&tx, "finding.deleted", id, "completed")?;
        tx.commit().map_err(db_error)
    }
    pub fn finding_compare(&self, a: &str, b: &str) -> Result<serde_json::Value> {
        let a = self.finding(a)?;
        let b = self.finding(b)?;
        if a.input.project_id != b.input.project_id {
            return Err(AppError::new(
                "PROJECT_MISMATCH",
                "Compare findings within one project",
            ));
        }
        Ok(serde_json::json!({"a":a,"b":b}))
    }
    pub fn finding_merge(&self, primary: &str, duplicate: &str) -> Result<serde_json::Value> {
        if primary == duplicate {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Choose two different findings",
            ));
        }
        let a = self.finding(primary)?;
        let b = self.finding(duplicate)?;
        if a.input.project_id != b.input.project_id
            || a.duplicate_of.is_some()
            || b.duplicate_of.is_some()
            || (a.input.target_id.is_some()
                && b.input.target_id.is_some()
                && a.input.target_id != b.input.target_id)
        {
            return Err(AppError::new(
                "MERGE_CONFLICT",
                "Choose non-duplicate findings in the same project and target",
            ));
        }
        let mut input = a.input.clone();
        input.evidence_ids.extend(b.input.evidence_ids.clone());
        input.evidence_ids.sort();
        input.evidence_ids.dedup();
        input.notes.push_str(&format!(
            "\nMerged evidence links from {}. Source finding content retained separately.",
            b.id
        ));
        self.validate_finding(&mut input, a.input.status == FindingStatus::Verified)?;
        let mut dup = b.input.clone();
        dup.status = FindingStatus::Duplicate;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        for (f, data, dupof) in [(&a, &input, None), (&b, &dup, Some(primary))] {
            let status = serde_json::to_value(data.status).map_err(json_error)?;
            if tx.execute("UPDATE findings SET data_json=?1,status=?2,duplicate_of=?3,revision=revision+1,updated_at=?4 WHERE id=?5 AND revision=?6",params![serde_json::to_string(data).map_err(json_error)?,status.as_str(),dupof,now(),f.id,f.revision]).map_err(db_error)?!=1 {return Err(AppError::new("STALE_RECORD","Finding changed during merge"));}
        }
        for id in &input.evidence_ids {
            tx.execute(
                "INSERT OR IGNORE INTO finding_evidence(finding_id,evidence_id) VALUES(?1,?2)",
                params![primary, id],
            )
            .map_err(db_error)?;
        }
        Self::audit(&tx, "finding.merged", duplicate, primary)?;
        tx.commit().map_err(db_error)?;
        self.finding_compare(primary, duplicate)
    }
}
