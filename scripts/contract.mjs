// Keeps contract/v1 a byte-identical mirror of the Lithe plugin contract. The
// contract is defined in the Lithe repository (shared/contracts/plugins/v1);
// contract.lock.json pins the upstream commit and the SHA-256 of every file.
//
//   node scripts/contract.mjs verify               offline: lock, hashes, $id, links, unpinned files
//   node scripts/contract.mjs verify --upstream    also compare with the directory at the pinned commit
//   node scripts/contract.mjs drift [--ref <ref>]  report what upstream has changed since the pin (default: preview)
//   node scripts/contract.mjs sync --revision <sha> [--checkout <Lithe clone>]
//
// GitHub API calls use GITHUB_TOKEN when it is set.
import { execFile } from 'node:child_process';
import { createHash } from 'node:crypto';
import { lstat, mkdir, readFile, readdir, rename, rm, writeFile } from 'node:fs/promises';
import { dirname, join, posix, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { promisify } from 'node:util';

export const repository = resolve(dirname(fileURLToPath(import.meta.url)), '..');
/** Where GitHub serves this repository's files; every schema's `$id` is its URL here. */
export const publishedBase = 'https://raw.githubusercontent.com/Lithe-IDEA/lithe-plugin-docs/main/';
/** The mirrored directory, relative to the repository root. */
export const mirror = 'contract/v1';
const mirrorParent = posix.dirname(mirror);
const lockName = 'contract.lock.json';
/**
 * The upstream branch contract changes merge into. Lithe's default branch is
 * the stable `main`, which only catches up at a release.
 */
export const developmentBranch = 'preview';
const fetchTimeoutMs = 30_000;

const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');

/** The object ID git gives these bytes, so upstream trees compare without downloads. */
export const gitBlobID = bytes =>
  createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex');

function isSafePath(path) {
  return typeof path === 'string' && !path.startsWith('/') && !path.includes('\\')
    && path.split('/').every(part => part && part !== '.' && part !== '..');
}

/**
 * Reads contract.lock.json and rejects anything but an exact, safe pin. With
 * `pinned: false` only the source repository and directory are required, so a
 * new mirror can start from a lock that names nothing else and run sync.
 */
export async function readLock(root = repository, { pinned = true } = {}) {
  const lock = JSON.parse(await readFile(join(root, lockName), 'utf8'));
  const source = lock.source ?? {};
  if (lock.schemaVersion !== 1 || !/^https:\/\/github\.com\/[A-Za-z0-9._-]+\/[A-Za-z0-9._-]+$/.test(source.repository)
    || !isSafePath(source.directory)) {
    throw new Error(`${lockName} must name an upstream GitHub repository and contract directory.`);
  }
  if (!pinned) return lock;
  if (!/^[0-9a-f]{40}$/.test(source.revision) || !Array.isArray(lock.files) || lock.files.length === 0) {
    throw new Error(`${lockName} must pin an exact upstream commit and file list.`);
  }
  const paths = new Set();
  for (const entry of lock.files) {
    if (!isSafePath(entry.path) || !/^[0-9a-f]{64}$/.test(entry.sha256) || paths.has(entry.path)) {
      throw new Error(`Invalid or duplicate ${lockName} entry: ${JSON.stringify(entry)}`);
    }
    paths.add(entry.path);
  }
  return lock;
}

/**
 * Content rules every mirrored file must meet, upstream included: a schema
 * names its own URL here as `$id`, and a Markdown link either has a scheme or
 * resolves to another mirrored file.
 */
function checkContent(path, bytes, paths) {
  if (path.endsWith('.schema.json')) {
    const expected = `${publishedBase}${mirror}/${path}`;
    const id = JSON.parse(bytes.toString('utf8')).$id;
    if (id !== expected) throw new Error(`${path}: $id is ${JSON.stringify(id)}, expected ${JSON.stringify(expected)}.`);
  }
  if (path.endsWith('.md')) {
    for (const [, target] of bytes.toString('utf8').matchAll(/\]\(([^)\s]+)\)/g)) {
      if (target.startsWith('#') || /^[a-z][a-z0-9+.-]*:/i.test(target)) continue;
      const linked = posix.normalize(posix.join(posix.dirname(path), target.split('#')[0]));
      if (!paths.has(linked)) {
        throw new Error(`${path}: link ${target} leaves the contract; upstream must use an absolute URL.`);
      }
    }
  }
}

/** Offline check of the mirror against the lock; returns the lock. */
export async function verify(root = repository) {
  const lock = await readLock(root);
  const paths = new Set(lock.files.map(entry => entry.path));
  for (const entry of lock.files) {
    const file = join(root, mirror, entry.path);
    if (!(await lstat(file)).isFile()) throw new Error(`Pinned file is not a regular file: ${mirror}/${entry.path}`);
    const bytes = await readFile(file);
    if (sha256(bytes) !== entry.sha256) {
      throw new Error(`SHA-256 mismatch: ${mirror}/${entry.path}. Change the contract upstream and sync instead.`);
    }
    checkContent(entry.path, bytes, paths);
  }
  const expected = new Set(lock.files.map(entry => `${mirror}/${entry.path}`));
  async function walk(directory, prefix) {
    for (const item of await readdir(directory, { withFileTypes: true })) {
      const path = `${prefix}/${item.name}`;
      if (item.isDirectory()) await walk(join(directory, item.name), path);
      else if (!expected.has(path)) throw new Error(`Unpinned file under ${mirrorParent}/: ${path}`);
    }
  }
  await walk(join(root, mirrorParent), mirrorParent);
  return lock;
}

/** Lists a git tree as `path → blob ID`, rejecting anything but plain files. */
function blobsFromTree(entries, describe) {
  const blobs = new Map();
  for (const { mode, type, sha, path } of entries) {
    if (type === 'tree') continue;
    if (type !== 'blob' || !['100644', '100755'].includes(mode) || !isSafePath(path)) {
      throw new Error(`${describe}: ${path} is not a plain file (${type} ${mode}).`);
    }
    blobs.set(path, sha);
  }
  if (blobs.size === 0) throw new Error(`${describe} has no files.`);
  return blobs;
}

/**
 * Reads the contract directory from GitHub: the tree through the REST API and
 * file bytes from raw.githubusercontent.com.
 */
export function githubSource({ repository: sourceRepository, directory }, {
  token = process.env.GITHUB_TOKEN, fetch: request = globalThis.fetch,
} = {}) {
  const slug = new URL(sourceRepository).pathname.slice(1);
  async function get(url, headers = {}) {
    const response = await request(url, { headers, signal: AbortSignal.timeout(fetchTimeoutMs) });
    if (!response.ok) throw new Error(`GET ${url} failed: HTTP ${response.status}`);
    return response;
  }
  return {
    async list(ref) {
      if (!/^[A-Za-z0-9._/-]+$/.test(ref)) throw new Error(`Unsupported upstream ref: ${ref}`);
      const url = `https://api.github.com/repos/${slug}/git/trees/${ref}:${directory}?recursive=1`;
      const headers = { accept: 'application/vnd.github+json', 'x-github-api-version': '2022-11-28' };
      if (token) headers.authorization = `Bearer ${token}`;
      const tree = await (await get(url, headers)).json();
      if (tree.truncated) throw new Error(`GitHub truncated the tree of ${directory} at ${ref}.`);
      return blobsFromTree(tree.tree, `${directory} at ${ref}`);
    },
    async read(path, revision) {
      const response = await get(`https://raw.githubusercontent.com/${slug}/${revision}/${directory}/${path}`);
      return Buffer.from(await response.arrayBuffer());
    },
  };
}

/** Reads the contract directory from a local clone of the Lithe repository. */
export function checkoutSource(checkout, { directory }) {
  const run = promisify(execFile);
  const git = (args, encoding = 'utf8') => run('git', ['-C', checkout, ...args],
    { encoding, maxBuffer: 16 * 1024 * 1024, timeout: fetchTimeoutMs });
  return {
    async list(ref) {
      const { stdout } = await git(['ls-tree', '-r', '-z', '--full-tree', ref, '--', `${directory}/`]);
      const entries = stdout.split('\0').filter(Boolean).map(line => {
        const [meta, path] = line.split('\t');
        const [mode, type, sha] = meta.split(' ');
        return { mode, type, sha, path: path.slice(directory.length + 1) };
      });
      return blobsFromTree(entries, `${directory} at ${ref}`);
    },
    async read(path, revision) {
      return (await git(['cat-file', 'blob', `${revision}:${directory}/${path}`], 'buffer')).stdout;
    },
  };
}

/** Compares the mirror with the upstream directory at `ref`. */
export async function compareUpstream(root, lock, source, ref) {
  const upstream = await source.list(ref);
  const local = new Map();
  for (const entry of lock.files) local.set(entry.path, gitBlobID(await readFile(join(root, mirror, entry.path))));
  return {
    added: [...upstream.keys()].filter(path => !local.has(path)).sort(),
    removed: [...local.keys()].filter(path => !upstream.has(path)).sort(),
    changed: [...upstream].filter(([path, id]) => local.has(path) && local.get(path) !== id).map(([path]) => path).sort(),
  };
}

function describeDifference({ added, removed, changed }) {
  return [['added', added], ['removed', removed], ['changed', changed]]
    .filter(([, paths]) => paths.length).map(([kind, paths]) => `${kind}: ${paths.join(', ')}`).join('; ');
}

/**
 * Replaces the mirror with the upstream directory at `revision` and pins it.
 * Everything is checked before the mirror changes; a crash between the swap and
 * the lock write leaves a mismatch that verify reports and a rerun repairs.
 */
export async function sync(root, source, revision) {
  if (!/^[0-9a-f]{40}$/.test(revision)) throw new Error('sync needs a full 40-character upstream commit.');
  const lock = await readLock(root, { pinned: false });
  const blobs = await source.list(revision);
  const paths = new Set(blobs.keys());
  const contents = [];
  for (const [path, id] of [...blobs].sort(([a], [b]) => (a < b ? -1 : 1))) {
    const bytes = await source.read(path, revision);
    if (gitBlobID(bytes) !== id) throw new Error(`Upstream ${path} does not match its tree entry at ${revision}.`);
    checkContent(path, bytes, paths);
    contents.push([path, bytes]);
  }
  const target = join(root, mirror);
  const staging = `${target}.${process.pid}.tmp`;
  const previous = `${target}.${process.pid}.old.tmp`;
  let movedAside = false;
  try {
    for (const [path, bytes] of contents) {
      await mkdir(dirname(join(staging, path)), { recursive: true });
      await writeFile(join(staging, path), bytes);
    }
    try {
      await rename(target, previous);
      movedAside = true;
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
    }
    await rename(staging, target);
    movedAside = false;
  } finally {
    if (movedAside) await rename(previous, target);
    await rm(staging, { recursive: true, force: true });
    await rm(previous, { recursive: true, force: true });
  }
  const next = {
    schemaVersion: 1,
    source: { repository: lock.source.repository, directory: lock.source.directory, revision },
    files: contents.map(([path, bytes]) => ({ path, sha256: sha256(bytes) })),
  };
  const temporary = join(root, `${lockName}.${process.pid}.tmp`);
  await writeFile(temporary, `${JSON.stringify(next, null, 2)}\n`);
  await rename(temporary, join(root, lockName));
  return next;
}

function option(args, name) {
  const index = args.indexOf(name);
  return index < 0 ? undefined : args[index + 1];
}

async function main(args) {
  const [command] = args;
  if (command === 'verify') {
    const lock = await verify();
    if (args.includes('--upstream')) {
      const difference = describeDifference(
        await compareUpstream(repository, lock, githubSource(lock.source), lock.source.revision));
      if (difference) throw new Error(`Mirror differs from ${lock.source.revision}: ${difference}`);
    }
    console.log(`Verified ${lock.files.length} files pinned to ${lock.source.repository}@${lock.source.revision}`);
  } else if (command === 'drift') {
    const lock = await verify();
    const ref = option(args, '--ref') ?? developmentBranch;
    const difference = describeDifference(await compareUpstream(repository, lock, githubSource(lock.source), ref));
    if (difference) {
      throw new Error(`Upstream ${ref} changed the contract (${difference}). Run: node scripts/contract.mjs sync --revision <commit>`);
    }
    console.log(`No drift: upstream ${ref} matches ${lock.source.revision}`);
  } else if (command === 'sync') {
    const lock = await readLock(repository, { pinned: false });
    const checkout = option(args, '--checkout');
    const source = checkout ? checkoutSource(resolve(checkout), lock.source) : githubSource(lock.source);
    const next = await sync(repository, source, option(args, '--revision') ?? '');
    await verify();
    console.log(`Synced ${next.files.length} files from ${next.source.repository}@${next.source.revision}`);
  } else {
    throw new Error('Usage: node scripts/contract.mjs verify [--upstream] | drift [--ref <ref>] | sync --revision <sha> [--checkout <dir>]');
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main(process.argv.slice(2)).catch(error => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
