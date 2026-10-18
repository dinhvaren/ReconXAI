use serde_json::Value;

#[test]
fn shipped_workflows_are_valid_yaml_and_commands_remain_strings() {
    for (name, source) in [
        ("ci.yml", include_str!("../../../.github/workflows/ci.yml")),
        (
            "codeql.yml",
            include_str!("../../../.github/workflows/codeql.yml"),
        ),
    ] {
        let value: Value = serde_saphyr::from_str(source)
            .unwrap_or_else(|error| panic!("Invalid workflow YAML in {name}: {error}"));
        let jobs = value["jobs"]
            .as_object()
            .expect("Workflow jobs must be a map");
        for job in jobs.values() {
            for step in job["steps"].as_array().expect("Steps must be an array") {
                if let Some(command) = step.get("run") {
                    assert!(
                        command.is_string(),
                        "{name}: a run command was parsed as a mapping instead of a string"
                    );
                }
            }
        }
    }
}
