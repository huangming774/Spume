# 配置 QStash 定时任务（第一期：同步 + 维护 + 索引 + 模型轮换）。
#
# 用法：
#   $env:QSTASH_TOKEN = "<Upstash QStash token>"
#   ./deploy/qstash-schedules.ps1 [-BaseUrl "https://www.miku.coffee"]
#
# 前置条件（Vercel 环境变量）：
#   QSTASH_CURRENT_SIGNING_KEY / QSTASH_NEXT_SIGNING_KEY（签名校验）
#   ACG_PUBLIC_ORIGIN（如 https://www.miku.coffee，核对调度目标）
#
# 幂等：先列出已有 schedule，目标 URL 相同则跳过，不重复创建。
# 未配置 QStash 时后端行为不变（手动 /api/v1/sync/trigger 仍可用）。

param(
    [string]$BaseUrl = "https://www.miku.coffee"
)

$ErrorActionPreference = "Stop"
$token = $env:QSTASH_TOKEN
if (-not $token) {
    Write-Error "请先设置环境变量 QSTASH_TOKEN（Upstash 控制台 → QStash → Request Builder 里的 token）"
}

$headers = @{ Authorization = "Bearer $token" }
$api = "https://qstash.upstash.io/v2"

# 目标路径 => cron（UTC）。bili 每小时、radar 每 30 分钟、维护每日 03:00、
# 索引每日 03:20、模型轮换每日 03:40、壁纸同步每日 04:00。
$jobs = @(
    @{ path = "/api/v1/sync/trigger";        cron = "0 * * * *" },
    @{ path = "/api/internal/sync/radar";    cron = "*/30 * * * *" },
    @{ path = "/api/internal/maintenance";   cron = "0 3 * * *" },
    @{ path = "/api/internal/index-content"; cron = "20 3 * * *" },
    @{ path = "/api/internal/refresh-ai-model"; cron = "40 3 * * *" },
    @{ path = "/api/internal/sync/wallpaper"; cron = "0 4 * * *" }
)

$existing = @()
try {
    $existing = Invoke-RestMethod -Uri "$api/schedules" -Headers $headers -TimeoutSec 30
} catch {
    Write-Warning "列出现有 schedule 失败（首次使用可能为空）：$($_.Exception.Message)"
}

foreach ($job in $jobs) {
    $destination = "$BaseUrl$($job.path)"
    $dup = $existing | Where-Object { $_.destination -eq $destination }
    if ($dup) {
        Write-Host "skip  $($job.cron)  $destination (已存在)"
        continue
    }
    $createHeaders = @{
        Authorization     = "Bearer $token"
        "Upstash-Cron"    = $job.cron
        "Upstash-Retries" = "3"
        "Content-Type"    = "application/json"
    }
    $result = Invoke-RestMethod -Uri "$api/schedules/$destination" -Method Post -Headers $createHeaders -Body "{}" -TimeoutSec 30
    Write-Host "created $($job.cron)  $destination  scheduleId=$($result.scheduleId)"
}

Write-Host "完成。可用 Invoke-RestMethod -Uri '$api/schedules' -Headers @{Authorization='Bearer <token>'} 查看。"
