import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { copyWorkingTree, includesNativeWorkflows } from './sdk-e2e.mjs';

test('the pinned contract turns native workflow coverage on after protocol 5 is published', () => {
  const schema = versions => ({ properties: { requires: { properties: { buildProviders: { enum: versions } } } } });
  assert.equal(includesNativeWorkflows(schema([1])), false);
  assert.equal(includesNativeWorkflows(schema([1]), true), true);
  assert.equal(includesNativeWorkflows(schema([1, 2, 3, 4])), false);
  assert.equal(includesNativeWorkflows(schema([1, 2, 3, 4, 5])), true);
  assert.throws(() => includesNativeWorkflows({}), /enumerate/);
  assert.throws(() => includesNativeWorkflows(schema(['4'])), /enumerate/);
});

function git(directory, ...args) {
  return execFileSync('git', ['-C', directory, ...args], { timeout: 5000, encoding: 'utf8',
    env: { ...process.env, GIT_OPTIONAL_LOCKS: '0' } });
}

test('explicit working-tree snapshots include staged, unstaged and untracked inputs without changing the source', { timeout: 20000 }, async (t) => {
  const root = await mkdtemp(join(tmpdir(), 'sdk-working-tree-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const source = join(root, 'source');
  const destination = join(root, 'destination');
  await mkdir(join(source, 'rust'), { recursive: true });
  await mkdir(join(source, 'shared'));
  await writeFile(join(source, '.gitignore'), '/rust/target/\n');
  await writeFile(join(source, 'rust/input.rs'), 'original\n');
  await writeFile(join(source, 'rust/deleted.rs'), 'removed\n');
  await writeFile(join(source, 'shared/fixture.json'), '{}\n');
  await writeFile(join(source, 'outside.txt'), 'original\n');
  git(source, 'init', '-q');
  git(source, 'config', 'core.autocrlf', 'false');
  git(source, 'add', '.');
  git(source, '-c', 'user.name=SDK Test', '-c', 'user.email=sdk@example.invalid', '-c', 'commit.gpgsign=false',
    '-c', 'core.hooksPath=/dev/null', 'commit', '-qm', 'fixture');
  git(root, 'clone', '-q', '--no-hardlinks', '-c', 'core.autocrlf=false', source, destination);
  await writeFile(join(source, 'rust/input.rs'), 'staged\n');
  git(source, 'add', 'rust/input.rs');
  await writeFile(join(source, 'rust/input.rs'), 'unstaged final\n');
  await rm(join(source, 'rust/deleted.rs'));
  await writeFile(join(source, 'shared/new fixture.json'), '{"new":true}\n');
  await writeFile(join(source, 'outside.txt'), 'unrelated\n');
  await mkdir(join(source, 'rust/target'));
  await writeFile(join(source, 'rust/target/cache'), 'not source\n');
  const indexBefore = await readFile(join(source, '.git/index'));
  const before = git(source, 'status', '--porcelain');
  await copyWorkingTree(source, destination);
  assert.equal(await readFile(join(destination, 'rust/input.rs'), 'utf8'), 'unstaged final\n');
  assert.equal(await readFile(join(destination, 'shared/new fixture.json'), 'utf8'), '{"new":true}\n');
  await assert.rejects(readFile(join(destination, 'rust/deleted.rs')), { code: 'ENOENT' });
  await assert.rejects(readFile(join(destination, 'rust/target/cache')), { code: 'ENOENT' });
  assert.equal(await readFile(join(destination, 'outside.txt'), 'utf8'), 'original\n');
  assert.equal(git(source, 'status', '--porcelain'), before);
  assert.deepEqual(await readFile(join(source, '.git/index')), indexBefore);
  assert.equal(await readFile(join(source, 'rust/input.rs'), 'utf8'), 'unstaged final\n');
  await assert.rejects(copyWorkingTree(join(source, 'rust'), destination), /repository root/);
});
