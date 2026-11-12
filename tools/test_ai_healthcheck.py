import datetime as dt
import json
import unittest

from tools import ai_healthcheck


class AiHealthcheckTests(unittest.TestCase):
    def test_generate_schedule_returns_sorted_unique_times_in_range(self):
        times = ai_healthcheck.generate_schedule(
            count=20,
            start="08:00",
            end="23:30",
            seed=20260604,
        )

        self.assertEqual(len(times), 20)
        self.assertEqual(times, sorted(times))
        self.assertEqual(len(set(times)), 20)
        for value in times:
            self.assertGreaterEqual(value, dt.time(8, 0))
            self.assertLessEqual(value, dt.time(23, 30))

    def test_build_payload_uses_lightweight_fixed_context(self):
        payload = ai_healthcheck.build_payload("请回复 OK，并说明当前服务正常")

        self.assertEqual(payload["message"], "请回复 OK，并说明当前服务正常")
        self.assertEqual(payload["pageUrl"], "https://taozhiyy.top/")
        self.assertEqual(payload["pageTitle"], "AI Healthcheck")
        self.assertEqual(payload["pageContext"]["siteSection"], "healthcheck")
        self.assertIn("healthcheck", payload["pageContext"]["visibleText"])

    def test_build_headers_identifies_healthcheck_and_includes_cookie_when_set(self):
        headers = ai_healthcheck.build_headers("session_id=abc")

        self.assertEqual(headers["User-Agent"], "taozhiyy-ai-healthcheck")
        self.assertEqual(headers["Accept"], "application/json")
        self.assertEqual(headers["Content-Type"], "application/json")
        self.assertEqual(headers["Cookie"], "session_id=abc")

    def test_build_headers_omits_cookie_when_empty(self):
        headers = ai_healthcheck.build_headers("")

        self.assertNotIn("Cookie", headers)

    def test_post_healthcheck_sends_expected_request(self):
        calls = []

        class FakeResponse:
            status = 200

            def read(self):
                return json.dumps({"reply": "OK 服务正常"}).encode("utf-8")

            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc, tb):
                return False

        def fake_opener(request, timeout):
            calls.append((request, timeout))
            return FakeResponse()

        result = ai_healthcheck.post_healthcheck(
            endpoint="https://taozhiyy.top/api/chat",
            message="请回复 OK，并说明当前服务正常",
            cookie="session_id=abc",
            timeout=12,
            opener=fake_opener,
        )

        self.assertTrue(result["ok"])
        self.assertEqual(result["status"], 200)
        self.assertEqual(calls[0][1], 12)
        request = calls[0][0]
        self.assertEqual(request.full_url, "https://taozhiyy.top/api/chat")
        self.assertEqual(request.get_method(), "POST")
        self.assertEqual(request.get_header("User-agent"), "taozhiyy-ai-healthcheck")
        self.assertEqual(request.get_header("Cookie"), "session_id=abc")
        body = json.loads(request.data.decode("utf-8"))
        self.assertEqual(body["message"], "请回复 OK，并说明当前服务正常")

    def test_plan_accepts_seed_after_subcommand(self):
        exit_code = ai_healthcheck.main(["plan", "--seed", "20260604"])

        self.assertEqual(exit_code, 0)


if __name__ == "__main__":
    unittest.main()
