import unittest

from dependency_readiness import assess


class DependencyGateTests(unittest.TestCase):
    def setUp(self):
        self.github = {"status": "RETRIEVED", "alerts": []}
        self.npm = {"metadata": {"vulnerabilities": {"total": 0}}}
        self.cargo = {"vulnerabilities": {"found": False, "list": []}, "warnings": {}}

    def test_cargo_success_exit_does_not_hide_unsound_warning(self):
        self.cargo["warnings"]["unsound"] = [{"advisory": {"id": "FIXTURE"}, "package": {"name": "fixture", "version": "1.0.0"}}]
        result = assess(self.github, self.npm, self.cargo, {})
        self.assertEqual(result["dependency_gate"], "BLOCKED")
        self.assertIn("unsound", " ".join(result["blockers"]))

    def test_failed_scans_never_mean_zero_findings(self):
        result = assess({"status": "BLOCKED"}, {"error": {"code": "FIXTURE"}}, {}, {})
        self.assertEqual(len(result["blockers"]), 3)

    def test_source_patch_is_not_remote_alert_closure(self):
        self.github["alerts"] = [{"number": 2, "ghsa_id": "FIXTURE", "state": "open", "severity": "high", "dependency": {"package": {"ecosystem": "pip", "name": "fixture"}, "manifest_path": "requirements.txt"}, "first_patched_version": {"identifier": "6.19.0"}}]
        result = assess(self.github, self.npm, self.cargo, {("pip", "fixture"): ["6.19.0"]})
        self.assertEqual(result["alerts"][0]["source_state"], "PATCHED_IN_SOURCE")
        self.assertEqual(result["alerts"][0]["remote_state"], "open")
        self.assertFalse(result["release_ready"])

    def test_mixed_or_unknown_versions_still_block(self):
        self.github["alerts"] = [{"number": 1, "ghsa_id": "FIXTURE", "state": "open", "severity": "high", "dependency": {"package": {"ecosystem": "rust", "name": "fixture"}, "manifest_path": "Cargo.lock"}, "first_patched_version": {"identifier": "1.1.0"}}]
        for versions in [[], ["1.1.0-rc.1"], ["1.1.0", "1.0.9"]]:
            with self.subTest(versions=versions):
                result = assess(self.github, self.npm, self.cargo, {("rust", "fixture"): versions})
                self.assertEqual(result["dependency_gate"], "BLOCKED")


if __name__ == "__main__":
    unittest.main()
