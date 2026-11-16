/**
 * 合并构建：主站 SPA 已包含文章、建站日志和星空馆路由。
 *
 * 用法：
 *   node tools/vercel-build.mjs               # 完整构建（含 npm ci）
 *   node tools/vercel-build.mjs --skip-install # 跳过 npm ci，复用已有 node_modules（本地迭代用）
 *   node tools/vercel-build.mjs --only main    # 只构建主站后组装
 *
 * 最终产物：
 *   out/               ← main/dist（主站 SPA，含 /blog /build /projects）
 *   out/ai-assistant/  ← shared/ai-assistant
 */
import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readdirSync, rmSync, statSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const out = join(root, "out");

const argv = process.argv.slice(2);
const skipInstall = argv.includes("--skip-install");
const onlyIdx = argv.indexOf("--only");
const only = onlyIdx >= 0 ? (argv[onlyIdx + 1] || "").split(",").filter(Boolean) : null;
const should = (name) => !only || only.includes(name);

const isWin = process.platform === "win32";
const npmCmd = isWin ? "npm.cmd" : "npm";

function run(label, cmd, args, cwd) {
  console.log(`\n\x1b[36m▶ ${label}\x1b[0m`);
  console.log(`  $ ${cmd} ${args.join(" ")}  (cwd: ${cwd})`);
  const r = spawnSync(cmd, args, {
    cwd,
    stdio: "inherit",
    env: process.env,
    shell: false,
  });
  if (r.error) throw r.error;
  if (r.status !== 0) {
    throw new Error(`步骤失败: ${label}（退出码 ${r.status}）`);
  }
}

function npm(args, cwd) {
  run(`npm ${args.join(" ")}`, npmCmd, args, cwd);
}

function countFiles(dir) {
  let n = 0;
  for (const entry of readdirSync(dir)) {
    const p = join(dir, entry);
    n += statSync(p).isDirectory() ? countFiles(p) : 1;
  }
  return n;
}

function copy(label, from, to) {
  if (!existsSync(from)) {
    // --only 模式下允许只构建部分子站，缺失的产物跳过并提示
    if (only) {
      console.warn(`  跳过 ${label}：${from} 不存在（--only 模式）`);
      return;
    }
    throw new Error(`缺少构建产物：${from}（请先完成对应子站构建）`);
  }
  console.log(`  拷贝 ${label}: ${from} → ${to}`);
  cpSync(from, to, { recursive: true });
}

// 文章、建站日志、星空馆都在 main 的 SPA 路由里，不再单独产出 HTML 子站。
if (should("main")) {
  const dir = join(root, "main");
  const gallery = join(root, "previews", "project-gallery");
  if (!skipInstall) {
    npm(["ci"], dir);
    // 星空馆源码被 CosmosPage 直接 import，但 three/lucide 声明在它自己的 package.json 里，
    // 只能从 previews/project-gallery/node_modules 解析，所以必须单独安装。
    npm(["ci"], gallery);
  }
  npm(["run", "build"], dir);
}

// ---------- 5. 组装 out/ ----------
console.log("\n\x1b[36m▶ 组装 out/\x1b[0m");
rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

copy("主站 SPA", join(root, "main", "dist"), out);
copy("AI 助手", join(root, "shared", "ai-assistant"), join(out, "ai-assistant"));

// ---------- 校验 ----------
const required = [
  "index.html",
  join("covers", "taozhiyy.webp"),
  join("ai-assistant", "ai-assistant.js"),
];
const missing = required.filter((p) => !existsSync(join(out, p)));
if (missing.length && !only) {
  console.error("\n\x1b[31m✗ 校验失败，out/ 缺少：\x1b[0m");
  for (const m of missing) console.error(`  - ${m}`);
  process.exit(1);
}
if (missing.length) {
  console.warn("\n\x1b[33m! --only 模式，以下产物未生成：\x1b[0m");
  for (const m of missing) console.warn(`  - ${m}`);
}

console.log("\n\x1b[32m✓ 合并构建完成\x1b[0m");
console.log(`  输出目录: ${out}`);
console.log(`  文件总数: ${countFiles(out)}`);
console.log(`  本地预览: npx serve out  （注意 SPA fallback 需另配，见 vercel.json）`);
