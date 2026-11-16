import test from 'node:test';
import assert from 'node:assert/strict';
import { projects, filterProjects, nextProject, projectPage } from '../src/catalog.js';

test('the catalog starts empty so projects can be added later', () => {
  assert.deepEqual(projects, []);
  assert.equal(filterProjects(projects, '').length, 0);
  assert.equal(nextProject(projects, 'missing', 1), null);
  assert.equal(projectPage(projects).pages, 0);
});

test('search and paging still work for a supplied list', () => {
  const seed = { id: 'seed', title: 'Seed', summary: '', author: '', category: '', tags: [] };
  const many = Array.from({length:500}, (_, i) => ({...seed, id:`project-${i}`, title:`Project ${i}`}));
  assert.equal(projectPage(many, 0, 12).items.length, 12);
  assert.equal(projectPage(many, 41, 12).items.length, 8);
  assert.equal(projectPage(many, 0, 12).pages, 42);
  assert.equal(projectPage(many, 999, 12).page, 41);
  assert.equal(projectPage(many, -2, 12).page, 0);
  assert.equal(projectPage([], 0, 6).pages, 0);
  assert.equal(projectPage(many, 10, 12).items[0].id, 'project-120');
  assert.equal(filterProjects(many, 'Project 499').length, 1);
  assert.equal(many.length, 500);
});
