#!/usr/bin/env python3
"""Transparent AI healthcheck scheduler for taozhiyy.top."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import random
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Callable, Iterable


DEFAULT_ENDPOINT = "https://taozhiyy.top/api/chat"
DEFAULT_MESSAGE = "请回复 OK，并说明当前服务正常"
DEFAULT_UA = "taozhiyy-ai-healthcheck"
DEFAULT_COUNT = 20
DEFAULT_START = "08:00"
DEFAULT_END = "23:30"
DEFAULT_TIMEOUT = 20
DEFAULT_PAGE_URL = "https://taozhiyy.top/"
DEFAULT_LOG = "/var/log/taozhiyy-ai-healthcheck.log"


def parse_clock(value: str) -> dt.time:
    try:
        hour, minute = value.split(":", 1)
        return dt.time(int(hour), int(minute))
    except (TypeError, ValueError) as exc:
        raise argparse.ArgumentTypeError(f"invalid HH:MM time: {value}") from exc


def _minute_of_day(value: dt.time) -> int:
    return value.hour * 60 + value.minute


def _time_from_minute(value: int) -> dt.time:
    return dt.time(value // 60, value % 60)


def generate_schedule(
    count: int = DEFAULT_COUNT,
    start: str = DEFAULT_START,
    end: str = DEFAULT_END,
    seed: int | None = None,
) -> list[dt.time]:
    if count < 1:
        raise ValueError("count must be at least 1")

    start_min = _minute_of_day(parse_clock(start))
    end_min = _minute_of_day(parse_clock(end))
    if end_min < start_min:
        raise ValueError("end must be later than start")

    available = list(range(start_min, end_min + 1))
    if count > len(available):
        raise ValueError("count is larger than available minutes")

    rng = random.Random(seed)
    return sorted(_time_from_minute(value) for value in rng.sample(available, count))


def build_payload(message: str = DEFAULT_MESSAGE) -> dict[str, object]:
    return {
        "message": message,
        "pageUrl": DEFAULT_PAGE_URL,
        "pageTitle": "AI Healthcheck",
        "pageContext": {
            "pageUrl": DEFAULT_PAGE_URL,
            "pageTitle": "AI Healthcheck",
            "pagePath": "/",
            "siteSection": "healthcheck",
            "headings": ["AI Healthcheck"],
            "visibleText": "taozhiyy-ai-healthcheck transparent service probe",
        },
    }


def build_headers(cookie: str = "") -> dict[str, str]:
    headers = {
        "User-Agent": DEFAULT_UA,
        "Accept": "application/json",
        "Content-Type": "application/json",
    }
    if cookie.strip():
        headers["Cookie"] = cookie.strip()
    return headers


def post_healthcheck(
    endpoint: str = DEFAULT_ENDPOINT,
    message: str = DEFAULT_MESSAGE,
    cookie: str = "",
    timeout: int = DEFAULT_TIMEOUT,
    opener: Callable[[urllib.request.Request, int], object] | None = None,
) -> dict[str, object]:
    data = json.dumps(build_payload(message), ensure_ascii=False).encode("utf-8")
    request = urllib.request.Request(
        endpoint,
        data=data,
        headers=build_headers(cookie),
        method="POST",
    )
    open_request = opener or urllib.request.urlopen

    try:
        with open_request(request, timeout=timeout) as response:
            raw = response.read().decode("utf-8", errors="replace")
            parsed = json.loads(raw) if raw else {}
            status = getattr(response, "status", 200)
            return {
                "ok": 200 <= int(status) < 300 and not parsed.get("error"),
                "status": status,
                "reply": parsed.get("reply", ""),
                "error": parsed.get("error", ""),
                "remaining": parsed.get("remaining"),
            }
    except urllib.error.HTTPError as exc:
        raw = exc.read().decode("utf-8", errors="replace")
        try:
            parsed = json.loads(raw) if raw else {}
        except json.JSONDecodeError:
            parsed = {"message": raw}
        return {
            "ok": False,
            "status": exc.code,
            "error": parsed.get("error", "HTTP_ERROR"),
            "message": parsed.get("message", ""),
        }
    except Exception as exc:  # noqa: BLE001 - CLI logs all transport failures.
        return {"ok": False, "status": 0, "error": type(exc).__name__, "message": str(exc)}


def write_log(path: str, event: dict[str, object]) -> None:
    event = {"ts": dt.datetime.now(dt.timezone.utc).isoformat(), **event}
    log_path = Path(path)
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(event, ensure_ascii=False, sort_keys=True) + "\n")


def _env_int(name: str, default: int) -> int:
    value = os.getenv(name, "").strip()
    if not value:
        return default
    return int(value)


def _format_times(times: Iterable[dt.time]) -> list[str]:
    return [value.strftime("%H:%M") for value in times]


def run_once(args: argparse.Namespace) -> int:
    result = post_healthcheck(
        endpoint=args.endpoint,
        message=args.message,
        cookie=args.cookie,
        timeout=args.timeout,
    )
    write_log(args.log, {"mode": "run-once", **result})
    print(json.dumps(result, ensure_ascii=False, sort_keys=True))
    return 0 if result.get("ok") else 1


def run_day(args: argparse.Namespace) -> int:
    today_seed = args.seed
    if today_seed is None:
        today_seed = int(dt.date.today().strftime("%Y%m%d"))
    schedule = generate_schedule(args.count, args.start, args.end, today_seed)
    write_log(args.log, {"mode": "run-day-start", "schedule": _format_times(schedule)})

    failures = 0
    today = dt.date.today()
    for planned in schedule:
        target = dt.datetime.combine(today, planned)
        wait_seconds = (target - dt.datetime.now()).total_seconds()
        if wait_seconds > 0:
            time.sleep(wait_seconds)
        result = post_healthcheck(
            endpoint=args.endpoint,
            message=args.message,
            cookie=args.cookie,
            timeout=args.timeout,
        )
        write_log(args.log, {"mode": "run-day-check", "planned": planned.strftime("%H:%M"), **result})
        failures += 0 if result.get("ok") else 1
    write_log(args.log, {"mode": "run-day-finish", "failures": failures})
    return 0 if failures == 0 else 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Transparent taozhiyy AI healthcheck")
    parser.add_argument("--endpoint", default=os.getenv("TAOZIYY_AI_ENDPOINT", DEFAULT_ENDPOINT))
    parser.add_argument("--message", default=os.getenv("TAOZIYY_AI_MESSAGE", DEFAULT_MESSAGE))
    parser.add_argument("--cookie", default=os.getenv("TAOZIYY_AI_COOKIE", ""))
    parser.add_argument("--timeout", type=int, default=_env_int("TAOZIYY_AI_TIMEOUT", DEFAULT_TIMEOUT))
    parser.add_argument("--log", default=os.getenv("TAOZIYY_AI_LOG", DEFAULT_LOG))
    parser.add_argument("--count", type=int, default=_env_int("TAOZIYY_AI_COUNT", DEFAULT_COUNT))
    parser.add_argument("--start", default=os.getenv("TAOZIYY_AI_START", DEFAULT_START))
    parser.add_argument("--end", default=os.getenv("TAOZIYY_AI_END", DEFAULT_END))
    parser.add_argument("--seed", type=int, default=None)

    subparsers = parser.add_subparsers(dest="command", required=True)
    plan_parser = subparsers.add_parser("plan", help="print today's randomized schedule")
    plan_parser.add_argument("--seed", type=int, default=None)
    run_once_parser = subparsers.add_parser("run-once", help="send one healthcheck request")
    run_once_parser.add_argument("--seed", type=int, default=None)
    run_day_parser = subparsers.add_parser("run-day", help="run all checks for the generated daily schedule")
    run_day_parser.add_argument("--seed", type=int, default=None)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.command == "plan":
        seed = args.seed
        if seed is None:
            seed = int(dt.date.today().strftime("%Y%m%d"))
        for value in _format_times(generate_schedule(args.count, args.start, args.end, seed)):
            print(value)
        return 0
    if args.command == "run-once":
        return run_once(args)
    if args.command == "run-day":
        return run_day(args)
    parser.error(f"unknown command: {args.command}")
    return 2


if __name__ == "__main__":
    sys.exit(main())
