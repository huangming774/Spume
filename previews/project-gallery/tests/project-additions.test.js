import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { projects } from '../src/catalog.js';

const additions = ['localsend', 'nature-skills', 'anti-defensive-writing'];

test('static count metadata agrees with an empty project list before JavaScript runs', () => {
  const html = readFileSync(new URL('../index.html', import.meta.url), 'utf8');
  const realCount = projects.filter(p => !p.isGuide).length;
  assert.equal(realCount, 0);
  assert.ok(html.includes('class="collection-count">PROJECT INDEX / 00 + GUIDE</span>'));
  assert.ok(html.includes('aria-label="0 个项目与板块介绍"'));
  assert.ok(html.includes('<strong>00 <i>/</i> 00</strong>'));
});

test('curated sample projects are not bundled in the empty catalog', () => {
  for (const id of additions) {
    assert.equal(projects.find(p => p.id === id), undefined);
  }
});
