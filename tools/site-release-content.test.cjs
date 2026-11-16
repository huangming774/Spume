const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '..');
const read = (name) => fs.readFileSync(path.join(root, name), 'utf8');

test('journal content starts with no articles', () => {
  for (const file of ['build/src/data/content.json', 'main/src/subapps/journal/src/data/content.json']) {
    const data = JSON.parse(read(file));
    assert.deepEqual(data.articles, []);
  }
  assert.equal(fs.readdirSync(path.join(root, 'build/content/articles')).filter((name) => name.endsWith('.md')).length, 0);
});

test('console changelog has no sample updates and reads published entries', async () => {
  const { CONSOLE_UPDATES } = await import('../main/src/components/site-console/consoleContent.js');
  assert.deepEqual(CONSOLE_UPDATES, []);
  const ui = read('main/src/components/site-console/SiteConsole.jsx');
  assert.match(ui, /loadRecentEntries/);
  assert.match(ui, /还没有更新。/);
});

test('lottery timeline summary reflects the current static image source', () => {
  const source = read('build/src/data/chronicle.ts');
  const entry = source.split("'site-growth-16-lottery-wallpaper-source': {")[1].split("'site-growth-15-time-archive':")[0];
  assert.match(entry, /t\.alcy\.cc/);
  assert.doesNotMatch(entry, /换成 Lolicon/);
});
