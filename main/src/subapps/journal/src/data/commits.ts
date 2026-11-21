import type { ArchiveCategory, ChronicleEntry } from './chronicle';

/** 后端 /api/client/repo-commits 裁剪后的单条提交。 */
export type RepoCommit = {
  sha: string;
  shortSha: string;
  title: string;
  body: string;
  kind: string;
  date: string;
  author: string;
  url: string;
};

export type CommitFeed = {
  configured: boolean;
  available: boolean;
  reason?: string;
  repo?: string;
  branch?: string;
  commits: RepoCommit[];
};

/** 约定式提交前缀 → 档案分类与双语标签。 */
const kindMeta: Record<string, { category: ArchiveCategory; label: [string, string] }> = {
  feat: { category: 'feature', label: ['功能生长', 'Feature'] },
  fix: { category: 'engineering', label: ['修复打磨', 'Fix'] },
  perf: { category: 'engineering', label: ['性能优化', 'Perf'] },
  refactor: { category: 'engineering', label: ['幕后工程', 'Refactor'] },
  style: { category: 'design', label: ['设计手记', 'Style'] },
  docs: { category: 'engineering', label: ['文档记录', 'Docs'] },
  test: { category: 'engineering', label: ['测试验证', 'Test'] },
  build: { category: 'engineering', label: ['构建发布', 'Build'] },
  ci: { category: 'engineering', label: ['流水线', 'CI'] },
  chore: { category: 'engineering', label: ['日常维护', 'Chore'] },
  revert: { category: 'engineering', label: ['回退记录', 'Revert'] },
};

const fallbackMeta = { category: 'feature' as ArchiveCategory, label: ['更新记录', 'Update'] as [string, string] };

export function commitCategoryLabel(kind: string): [string, string] {
  return (kindMeta[kind] ?? fallbackMeta).label;
}

/** 去掉标题里的约定式前缀，展示更干净的中文说明。 */
function cleanTitle(title: string): string {
  return title.replace(/^[a-z]+(\([^)]*\))?!?:\s*/i, '').trim() || title;
}

/** 提交时间（ISO）→ 本地日期 YYYY-MM-DD；解析失败退回原值前 10 位。 */
function commitDate(iso: string): string {
  const parsed = new Date(iso);
  if (Number.isNaN(parsed.getTime())) return iso.slice(0, 10);
  const month = String(parsed.getMonth() + 1).padStart(2, '0');
  const day = String(parsed.getDate()).padStart(2, '0');
  return `${parsed.getFullYear()}-${month}-${day}`;
}

/**
 * 把提交列表映射成时间轴条目，与手写成长记录同一形状。
 * number 用短哈希，和手写记录的序号区分开；提交没有独立详情页，
 * bodyHtml 留空，页面按 slug 前缀识别后外链到 GitHub。
 */
export function commitsToEntries(commits: RepoCommit[]): ChronicleEntry[] {
  return commits
    .filter(commit => commit.sha && commit.title)
    .map(commit => {
      const meta = kindMeta[commit.kind] ?? fallbackMeta;
      const title = cleanTitle(commit.title);
      const date = commitDate(commit.date);
      const summary = commit.body || (commit.author ? `${commit.author} · ${commit.shortSha}` : commit.shortSha);
      return {
        slug: `commit-${commit.shortSha}`,
        date,
        title_zh: title,
        title_en: title,
        excerpt_zh: commit.body,
        excerpt_en: commit.body,
        words: '',
        reads: '',
        minutes: '',
        bodyHtmlZh: '',
        bodyHtmlEn: '',
        category: meta.category,
        title: [title, title] as [string, string],
        tags: [commit.kind || 'commit', commit.shortSha].filter(Boolean),
        note: [summary, summary] as [string, string],
        number: commit.shortSha,
        commitUrl: commit.url,
        commitAuthor: commit.author,
      };
    });
}
