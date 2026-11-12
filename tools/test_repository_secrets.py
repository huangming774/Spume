from pathlib import Path
import re
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_PATH = ROOT / ".github" / "workflows" / "deploy.yml"


def git(*args):
    return subprocess.run(
        ["git", *args],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


class RepositorySecretBaselineTests(unittest.TestCase):
    def test_only_env_examples_are_tracked(self):
        result = git("ls-files")
        self.assertEqual(result.returncode, 0, result.stderr)

        env_file = re.compile(r"(^|/)[^/]*\.env($|\.)", re.IGNORECASE)
        bad = [
            path
            for path in result.stdout.splitlines()
            if env_file.search(path.replace("\\", "/"))
            and not path.lower().endswith(".env.example")
        ]

        self.assertEqual([], bad, "tracked env files must be examples only")

    def test_no_private_key_material_is_tracked(self):
        patterns = [
            "BEGIN " + "OPENSSH PRIVATE KEY",
            "BEGIN " + "RSA PRIVATE KEY",
            "BEGIN " + "DSA PRIVATE KEY",
            "BEGIN " + "EC PRIVATE KEY",
            "BEGIN " + "PRIVATE KEY",
        ]

        for pattern in patterns:
            result = git("grep", "-Il", pattern, "--", ".")
            if result.returncode == 1:
                continue
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual("", result.stdout.strip(), f"private key material found for {pattern!r}")

    def test_workflow_does_not_echo_secret_values(self):
        text = WORKFLOW_PATH.read_text(encoding="utf-8")

        self.assertNotIn("echo \"${{ secrets.", text)
        for line_no, line in enumerate(text.splitlines(), 1):
            if "echo" not in line:
                continue
            self.assertNotRegex(
                line,
                r"\b(UCLOUD_SUDO_PASSWORD|AUTH_OWNER_PASSWORD|AUTH_OWNER_SECURITY_ANSWER|"
                r"AGNES_API_KEY|DASHSCOPE_API_KEY|TENCENT_COS_SECRET|OWNER_PUBLISH_GITHUB_TOKEN|"
                r"SMTP_PASS|SYNC_TRIGGER_TOKEN)\b",
                f"sensitive variable echoed on line {line_no}",
            )


if __name__ == "__main__":
    unittest.main()
