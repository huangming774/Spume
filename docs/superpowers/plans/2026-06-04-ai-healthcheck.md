# AI Healthcheck Scheduler Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a transparent daily AI healthcheck scheduler that can call the site AI endpoint 20 times at randomized daily times.

**Architecture:** A focused Python CLI handles schedule generation and one-shot requests. A systemd service/timer template runs the daily scheduler on UCloud, with secrets provided by an environment file.

**Tech Stack:** Python standard library, `unittest`, `systemd`.

---

### Task 1: Python Tool And Tests

**Files:**
- Create: `tools/ai_healthcheck.py`
- Create: `tools/test_ai_healthcheck.py`

- [ ] Write tests for schedule count, time range, request body, User-Agent, and optional Cookie header.
- [ ] Run `python -m unittest tools.test_ai_healthcheck` and verify the tests fail before implementation.
- [ ] Implement `tools/ai_healthcheck.py` with `plan`, `run-once`, and `run-day`.
- [ ] Run `python -m unittest tools.test_ai_healthcheck` and verify the tests pass.

### Task 2: Systemd Templates

**Files:**
- Create: `deploy/ai-healthcheck.service`
- Create: `deploy/ai-healthcheck.timer`
- Create: `deploy/ai-healthcheck.env.example`

- [ ] Add a service that runs `python3 /opt/taozhiyy-tools/ai_healthcheck.py run-day`.
- [ ] Add a daily timer with a broad `RandomizedDelaySec`.
- [ ] Add an env example documenting `TAOZIYY_AI_COOKIE`, `TAOZIYY_AI_ENDPOINT`, `TAOZIYY_AI_COUNT`, and `TAOZIYY_AI_LOG`.

### Task 3: Verification And Commit

**Files:**
- Modify: git index only

- [ ] Run `python -m unittest tools.test_ai_healthcheck`.
- [ ] Run `python tools/ai_healthcheck.py plan --seed 20260604` and check it prints 20 times.
- [ ] Run `git status --short`.
- [ ] Commit with `feat: add transparent ai healthcheck scheduler`.
