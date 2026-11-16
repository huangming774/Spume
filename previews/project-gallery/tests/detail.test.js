import test from 'node:test';
import assert from 'node:assert/strict';
import { renderProjectDetail } from '../src/detail.js';

const sample = {id:'sample',title:'Sample <script>alert(1)</script>',author:'Author',category:'动画',summary:'Summary',source:'https://github.com/author/project',url:'https://example.com/',docs:'https://example.com/docs',license:'MIT',licenseUrl:'https://example.com/license',reviewedAt:'2026-10-28',cover:'/covers/sample.webp',tags:['Tag'],demo:{kind:'video',url:'https://example.com/demo.mp4',label:'作者演示',note:'Recorded by author'},sections:[['安装','在本地终端执行','npm install <package>']]};
test('details safely present code, authors, licenses, source and playable author demos', () => {
  const html = renderProjectDetail(sample);
  assert.ok(html.includes('&lt;script&gt;'));
  assert.ok(!html.includes('<script>'));
  assert.ok(html.includes('npm install &lt;package&gt;'));
  assert.ok(html.includes('data-copy-section="0"'));
  assert.ok(html.includes('Author'));
  assert.ok(html.includes('MIT'));
  assert.ok(html.includes('https://github.com/author/project'));
  assert.ok(html.includes('<video'));
  assert.ok(html.includes('preload="none"'));
  assert.ok(!html.includes('autoplay'));
  assert.ok(!html.includes('我的作品') && !html.includes('精选分享'));
});
test('external trials do not mount hidden remote iframes or accept non-HTTPS links', () => {
  const html = renderProjectDetail({...sample,demo:{kind:'external',url:'javascript:alert(1)',label:'Try',note:'External trial'}});
  assert.ok(!html.includes('<iframe'));
  assert.ok(!html.includes('javascript:'));
  assert.ok(html.includes('noopener noreferrer'));
});

test('reader uses a compact masthead, introduction and responsive section navigation', () => {
  const html = renderProjectDetail(sample);
  assert.ok(html.includes('class="reader-masthead"'));
  assert.ok(html.includes('id="detail-introduction"'));
  assert.ok(html.includes('class="reader-overview"'));
  assert.ok(html.includes('id="reader-section-select"'));
  assert.ok(html.includes('data-reader-link'));
  assert.ok(html.indexOf('id="detail-introduction"') < html.indexOf('id="detail-demo"'));
  assert.ok(html.includes('loading="lazy"'));
  assert.ok(html.includes('原作者'));
});

test('the introduction renderer keeps a guide entry free of repository and demo claims', () => {
  const guide = {
    id: 'project-guide',
    isGuide: true,
    title: '项目星空 · 板块介绍',
    author: 'spume',
    category: '板块介绍',
    summary: '认识奇想馆的项目分享空间。',
    cover: '/covers/project-guide.webp',
    reviewedAt: '2026-10-31',
    tags: ['项目星空', '板块介绍'],
    sections: [['这里是做什么的', '项目星空是奇想馆的项目分享空间，收录开源项目、实用工具、教程、试用、作者和来源。']],
  };
  const html = renderProjectDetail(guide);
  assert.match(html, /ABOUT THIS SPACE/);
  assert.match(html, /板块介绍/);
  assert.match(html, /项目星空页面实景/);
  assert.match(html, /编写者/);
  assert.match(html, /data-guide-browse/);
  assert.match(html, /浏览项目目录/);
  assert.doesNotMatch(html, /<a\b[^>]*>(?:源仓库|官方文档)|查看原始许可文件|id="detail-demo"|id="detail-source"|官方页面截图/);
  assert.equal((html.match(/data-reader-link/g) || []).length, guide.sections.length);
});
