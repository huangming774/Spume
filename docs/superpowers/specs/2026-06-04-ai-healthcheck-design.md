# AI Healthcheck Scheduler Design

## Goal

Create a transparent daily AI healthcheck that sends 20 lightweight requests to the site AI endpoint at randomized times between 08:00 and 23:30.

## Scope

- Requests target `https://taozhiyy.top/api/chat` by default.
- Each request uses the fixed prompt: `请回复 OK，并说明当前服务正常`.
- Requests identify themselves with `User-Agent: taozhiyy-ai-healthcheck`.
- The script reads an optional login cookie from `TAOZIYY_AI_COOKIE` so the checks can use the logged-in quota.
- Healthcheck calls are intentionally not excluded from existing AI statistics.
- Secrets are not committed. Cookie values live only in the server environment file.

## Architecture

The repository gets a small Python tool under `tools/`. The tool has two modes:

- `plan`: generate 20 randomized times for the current day, one per line.
- `run-once`: send one healthcheck request, log a JSON result, and exit.

UCloud runs a `systemd` service that schedules the 20 checks for the day by sleeping until each generated time and calling `run-once`. A timer starts that service once per day.

## Safety

- The script uses a fixed, honest `User-Agent`.
- The script sets a request timeout.
- The script does not retry aggressively.
- The script does not bypass quota or rate limits.
- The script does not store cookies or secrets in git.
