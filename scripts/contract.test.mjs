import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { test } from 'node:test';
import {
  checkoutSource, compareUpstream, gitBlobID, githubSource, mirror, publishedBase, readLock, sync, verify,
} from './contract.mjs';

const revision = 'a'.repeat(40);
const nextRevision = 'b'.repeat(40);
const directory = 'shared/contracts/plugins/v1';
const schemaName = 'plugin-manifest.schema.json';
const options = { timeout: 10_000 };

const schema = (id = `${publishedBase}${mirror}/${schemaName}`, type = 'object') =>
  Buffer.from(`${JSON.stringify({ $id: id, type }, null, 2)}\n`);
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');

/** The upstream directory every test starts from. */
const baseFiles = () => new Map([
  ['README.md', Buffer.from('# Contract\n\nSee [the fixture](fixtures/a.json) and [levels](#levels).\n')],
  [schemaName, schema()],
  ['fixtures/a.json', Buffer.from('{"a":1}\n')],
  ['fixtures/stale.json', Buffer.from('{}\n')],
]);

/** A temporary root that mirrors `files` at `revision`. */
function makeTree(t, files = baseFiles()) {
  const root = mkdtempSync(join(tmpdir(), 'lithe-plugin-docs-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const write = (path, data) => {
    mkdirSync(dirname(join(root, path)), { recursive: true });
    writeFileSync(join(root, path), data);
  };
  for (const [path, bytes] of files) write(`${mirror}/${path}`, bytes);
  write('contract.lock.json', JSON.stringify({
    schemaVersion: 1,
    source: { repository: 'https://github.com/1lck/Lithe-IDEA', directory, revision },
    files: [...files].map(([path, bytes]) => ({ path, sha256: sha256(bytes) })),
  }));
  return { root, write };
}

/** An in-memory upstream that serves `byRef[ref]`. */
function fakeSource(byRef) {
  return {
    async list(ref) {
      const files = byRef[ref];
      if (!files) throw new Error(`unknown ref ${ref}`);
      return new Map([...files].map(([path, bytes]) => [path, gitBlobID(bytes)]));
    },
    async read(path, ref) {
      return byRef[ref].get(path);
    },
  };
}

test('a pinned mirror verifies', options, async t => {
  const { root } = makeTree(t);
  assert.equal((await verify(root)).source.revision, revision);
});

test('an edited file is refused', options, async t => {
  const { root, write } = makeTree(t);
  write(`${mirror}/fixtures/a.json`, '{"a":2}\n');
  await assert.rejects(verify(root), /SHA-256 mismatch/);
});

test('a missing pinned file is refused', options, async t => {
  const { root } = makeTree(t);
  rmSync(join(root, mirror, 'fixtures/a.json'));
  await assert.rejects(verify(root), /ENOENT/);
});

test('a file under contract/ without a pin is refused', options, async t => {
  const { root, write } = makeTree(t);
  write('contract/v2/README.md', '# v2\n');
  await assert.rejects(verify(root), /Unpinned file under contract\/: contract\/v2\/README\.md/);
});

test('a schema whose $id is not its published URL is refused', options, async t => {
  const files = baseFiles();
  files.set(schemaName, schema(`https://lithe.dev/contracts/plugins/v1/${schemaName}`));
  const { root } = makeTree(t, files);
  await assert.rejects(verify(root), /\$id/);
});

test('a Markdown link that leaves the contract is refused', options, async t => {
  const files = baseFiles();
  files.set('README.md', Buffer.from('See [the core API](../../rust-core-api.md).\n'));
  const { root } = makeTree(t, files);
  await assert.rejects(verify(root), /leaves the contract/);
});

test('a lock that pins a branch instead of a commit is refused', options, async t => {
  const { root, write } = makeTree(t);
  const lock = JSON.parse(readFileSync(join(root, 'contract.lock.json'), 'utf8'));
  write('contract.lock.json', JSON.stringify({ ...lock, source: { ...lock.source, revision: 'preview' } }));
  await assert.rejects(verify(root), /exact upstream commit/);
  assert.equal((await readLock(root, { pinned: false })).source.directory, directory);
});

test('sync mirrors added, removed, and changed files and pins the commit', options, async t => {
  const { root } = makeTree(t);
  const next = baseFiles();
  next.delete('fixtures/stale.json');
  next.set('fixtures/a.json', Buffer.from('{"a":2}\n'));
  next.set('fixtures/marketplace/index.json', Buffer.from('{"schemaVersion":1}\n'));
  const source = fakeSource({ [revision]: baseFiles(), HEAD: next, [nextRevision]: next });

  assert.deepEqual(await compareUpstream(root, await verify(root), source, revision),
    { added: [], removed: [], changed: [] });
  assert.deepEqual(await compareUpstream(root, await verify(root), source, 'HEAD'), {
    added: ['fixtures/marketplace/index.json'], removed: ['fixtures/stale.json'], changed: ['fixtures/a.json'],
  });

  await sync(root, source, nextRevision);
  const lock = await verify(root);
  assert.equal(lock.source.revision, nextRevision);
  assert.equal(existsSync(join(root, mirror, 'fixtures/stale.json')), false);
  assert.deepEqual(readFileSync(join(root, mirror, 'fixtures/marketplace/index.json')),
    next.get('fixtures/marketplace/index.json'));
  assert.deepEqual(await compareUpstream(root, lock, source, 'HEAD'), { added: [], removed: [], changed: [] });
});

test('sync refuses an upstream schema with another $id and keeps the mirror', options, async t => {
  const { root } = makeTree(t);
  const next = baseFiles();
  next.set(schemaName, schema('https://example.com/x.json'));
  await assert.rejects(sync(root, fakeSource({ [nextRevision]: next }), nextRevision), /\$id/);
  assert.equal((await verify(root)).source.revision, revision);
});

test('sync refuses bytes that do not match the upstream tree', options, async t => {
  const { root } = makeTree(t);
  const source = fakeSource({ [nextRevision]: baseFiles() });
  const read = source.read;
  source.read = async (path, ref) => (path === 'fixtures/a.json' ? Buffer.from('tampered\n') : read(path, ref));
  await assert.rejects(sync(root, source, nextRevision), /does not match its tree entry/);
  assert.equal((await verify(root)).source.revision, revision);
});

test('the GitHub source lists plain files below the directory and sends the token', options, async () => {
  const calls = [];
  const request = async (url, init) => {
    calls.push({ url, authorization: init.headers.authorization });
    return {
      ok: true,
      json: async () => ({
        truncated: false,
        tree: [
          { path: 'README.md', mode: '100644', type: 'blob', sha: '1'.repeat(40) },
          { path: 'fixtures', mode: '040000', type: 'tree', sha: '2'.repeat(40) },
          { path: 'fixtures/a.json', mode: '100644', type: 'blob', sha: '3'.repeat(40) },
        ],
      }),
    };
  };
  const source = githubSource({ repository: 'https://github.com/1lck/Lithe-IDEA', directory },
    { token: 'test-token', fetch: request });
  assert.deepEqual([...await source.list('HEAD')], [['README.md', '1'.repeat(40)], ['fixtures/a.json', '3'.repeat(40)]]);
  assert.equal(calls[0].url, `https://api.github.com/repos/1lck/Lithe-IDEA/git/trees/HEAD:${directory}?recursive=1`);
  assert.equal(calls[0].authorization, 'Bearer test-token');
});

test('the GitHub source refuses symlinks and truncated trees', options, async () => {
  const respond = body => async () => ({ ok: true, json: async () => body });
  const config = { repository: 'https://github.com/1lck/Lithe-IDEA', directory };
  const link = githubSource(config, { token: '', fetch: respond({
    truncated: false, tree: [{ path: 'README.md', mode: '120000', type: 'blob', sha: '1'.repeat(40) }],
  }) });
  await assert.rejects(link.list('HEAD'), /not a plain file/);
  const truncated = githubSource(config, { token: '', fetch: respond({ truncated: true, tree: [] }) });
  await assert.rejects(truncated.list('HEAD'), /truncated/);
});

test('the checkout source reads a committed directory from a local clone', options, async t => {
  const clone = mkdtempSync(join(tmpdir(), 'lithe-clone-'));
  t.after(() => rmSync(clone, { recursive: true, force: true }));
  const git = (...args) => execFileSync('git', ['-C', clone, ...args], { encoding: 'utf8', timeout: 5_000 }).trim();
  git('init', '--quiet');
  for (const [path, bytes] of baseFiles()) {
    mkdirSync(dirname(join(clone, directory, path)), { recursive: true });
    writeFileSync(join(clone, directory, path), bytes);
  }
  writeFileSync(join(clone, 'outside.json'), '{}\n');
  git('add', '.');
  git('-c', 'user.name=Test', '-c', 'user.email=test@example.com', 'commit', '--quiet', '-m', 'contract');
  const commit = git('rev-parse', 'HEAD');

  const { root } = makeTree(t);
  const source = checkoutSource(clone, { directory });
  assert.deepEqual([...(await source.list(commit)).keys()].sort(), [...baseFiles().keys()].sort());
  await sync(root, source, commit);
  assert.equal((await verify(root)).source.revision, commit);
});
