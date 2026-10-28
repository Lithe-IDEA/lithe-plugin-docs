// Runs the SDK end-to-end scenario in sdk/e2e against Lithe's Rust Core.
//
//   node scripts/sdk-e2e.mjs [--ref <commit|branch>] [--source <Lithe git URL or path>]
//   node scripts/sdk-e2e.mjs --source <local Lithe root> --working-tree --build-workflows
//
// Core lives in the Lithe repository, so the scenario is copied into
// rust/lithe-core/tests/ of a disposable Lithe checkout in .artifacts/lithe
// (only rust/ and shared/ are checked out) and run there with cargo. By default
// that checkout is the commit contract.lock.json pins, the same Core the
// contract mirror came from; `--ref preview` tests the branch Lithe's changes
// merge into (its default branch, main, only moves at a release).
//
// The Node.js sample runs on the node running this script. The Java sample
// runs on LITHE_PLUGIN_SDK_JAVA_RUNTIME, or `java` on PATH, and is built with
// Maven unless LITHE_PLUGIN_SDK_JAVA_SAMPLE names a built sample-build.jar.
// The Rust sample backend is built with Cargo unless
// LITHE_PLUGIN_SDK_RUST_SAMPLE names a built sample-backend executable.
import { spawn } from 'node:child_process';
import { access, copyFile, mkdir, readdir, cp, realpath, readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { join, resolve } from 'node:path';
import { parseArgs } from 'node:util';
import { readLock, repository } from './contract.mjs';

const work = join(repository, '.artifacts', 'lithe');
const scenario = join(repository, 'sdk', 'e2e');
const tests = ['plugin_sdk_node', 'plugin_sdk_java', 'plugin_sdk_rust'];

/** Once the contract mirror includes protocol 6, the current native workflows are mandatory. */
export function includesNativeWorkflows(schema, requested = false) {
  const versions = schema?.properties?.requires?.properties?.buildProviders?.enum;
  if (!Array.isArray(versions) || versions.some(version => !Number.isInteger(version) || version < 1)) {
    throw new Error('The pinned contract must enumerate buildProviders versions.');
  }
  return requested || versions.includes(6);
}

function run(command, args, options = {}) {
  return new Promise((resolvePromise, reject) => {
    const child = spawn(command, args, { stdio: 'inherit', ...options });
    child.on('error', reject);
    child.on('close', (code, signal) => {
      if (code === 0) resolvePromise();
      else reject(new Error(`${command} ${args.join(' ')} failed (${signal ?? `exit ${code}`})`));
    });
  });
}

function output(command, args, trim = true) {
  return new Promise((resolvePromise, reject) => {
    const child = spawn(command, args, { stdio: ['ignore', 'pipe', 'inherit'],
      timeout: 15000, killSignal: 'SIGKILL', env: { ...process.env, GIT_OPTIONAL_LOCKS: '0' } });
    let text = '';
    child.stdout.on('data', chunk => { text += chunk; });
    child.on('error', reject);
    child.on('close', code => code === 0
      ? resolvePromise(trim ? text.trim() : text)
      : reject(new Error(`${command} ${args.join(' ')} failed (exit ${code})`)));
  });
}

async function which(command) {
  try {
    return await output('sh', ['-c', 'command -v "$1"', 'sh', command]);
  } catch {
    throw new Error(`${command} is required to run the Java SDK end-to-end check.`);
  }
}

/** Checks out `ref` from `source` into the disposable work directory, rust/ and shared/ only. */
async function checkoutCore(source, ref) {
  await mkdir(work, { recursive: true });
  await run('git', ['init', '-q', work]);
  await run('git', ['-C', work, 'sparse-checkout', 'set', '--cone', 'rust', 'shared']);
  // A named remote, because the partial clone fetches missing blobs from it later.
  await run('git', ['-C', work, 'config', 'remote.origin.url', source]);
  await run('git', ['-C', work, 'fetch', '-q', '--depth', '1', '--filter=blob:none', 'origin', ref]);
  await run('git', ['-C', work, 'checkout', '-q', '--force', '--detach', 'FETCH_HEAD']);
  // The checkout is disposable. Drop prior scenario/working-tree inputs so a
  // removed source cannot linger; ignored Cargo caches are left alone.
  await run('git', ['-C', work, 'clean', '-q', '-f', '-d', '--', 'rust', 'shared']);
  return output('git', ['-C', work, 'rev-parse', 'HEAD']);
}

/** Replays only rust/ and shared/ from an explicitly selected local working tree. */
export async function copyWorkingTree(source, destination) {
  const directory = await realpath(source);
  const root = await output('git', ['-C', directory, 'rev-parse', '--show-toplevel']);
  if (await realpath(root) !== directory) throw new Error('--working-tree source must be the repository root.');
  const patch = await output('git', ['-C', directory, 'diff', '--no-ext-diff', '--no-textconv', '--binary', 'HEAD', '--', 'rust', 'shared'], false);
  if (patch) {
    await new Promise((resolvePromise, reject) => {
      const child = spawn('git', ['-C', destination, 'apply', '--binary', '-'], {
        stdio: ['pipe', 'inherit', 'inherit'], timeout: 15000, killSignal: 'SIGKILL' });
      child.on('error', reject);
      child.stdin.on('error', reject);
      child.on('close', code => code === 0 ? resolvePromise() : reject(new Error('Cannot replay working-tree changes.')));
      child.stdin.end(patch);
    });
  }
  const files = (await output('git', ['-C', directory, 'ls-files', '--others', '--exclude-standard', '-z', '--', 'rust', 'shared'], false))
    .split('\0').filter(Boolean);
  for (const file of files) {
    const target = join(destination, file);
    await mkdir(resolve(target, '..'), { recursive: true });
    await cp(join(directory, file), target, { verbatimSymlinks: true });
  }
}

async function copyScenario(destination) {
  for (const entry of await readdir(scenario, { recursive: true, withFileTypes: true })) {
    if (!entry.isFile()) continue;
    const from = join(entry.parentPath ?? entry.path, entry.name);
    const to = join(destination, from.slice(scenario.length + 1));
    await mkdir(resolve(to, '..'), { recursive: true });
    await copyFile(from, to);
  }
}

async function javaSample() {
  if (process.env.LITHE_PLUGIN_SDK_JAVA_SAMPLE) return resolve(process.env.LITHE_PLUGIN_SDK_JAVA_SAMPLE);
  await run(await which('mvn'), ['-B', '-ntp', '-q', '-f', join(repository, 'sdk/java/pom.xml'),
    '-DskipTests', 'package']);
  const jar = join(repository, 'sdk/java/examples/sample-build/target/sample-build.jar');
  await access(jar);
  return jar;
}

async function rustSample() {
  if (process.env.LITHE_PLUGIN_SDK_RUST_SAMPLE) return resolve(process.env.LITHE_PLUGIN_SDK_RUST_SAMPLE);
  const manifest = join(repository, 'sdk/rust/Cargo.toml');
  await run('cargo', ['build', '--locked', '--quiet', '--manifest-path', manifest, '-p', 'sample-backend']);
  const executable = join(repository, 'sdk/rust/target/debug',
    process.platform === 'win32' ? 'sample-backend.exe' : 'sample-backend');
  await access(executable);
  return executable;
}

async function main() {
  const { values } = parseArgs({ options: { ref: { type: 'string' }, source: { type: 'string' },
    'working-tree': { type: 'boolean' }, 'build-workflows': { type: 'boolean' }, 'native-configuration': { type: 'boolean' } } });
  if (values['working-tree'] && (!values.source || values.ref)) {
    throw new Error('--working-tree requires a local --source and cannot be combined with --ref.');
  }
  const lock = await readLock();
  const schema = JSON.parse(await readFile(join(repository, 'contract/v1/plugin-manifest.schema.json'), 'utf8'));
  const configurationWorkflows = values['native-configuration'] || schema.properties.requires.properties.buildProviders.enum.includes(7);
  const buildWorkflows = includesNativeWorkflows(schema, values['build-workflows'] || configurationWorkflows);
  const ref = values['working-tree'] ? 'HEAD' : values.ref ?? lock.source.revision;
  const source = values.source ?? `${lock.source.repository}.git`;

  const java = process.env.LITHE_PLUGIN_SDK_JAVA_RUNTIME || await which('java');
  const jar = await javaSample();
  const nativeJar = process.env.LITHE_PLUGIN_SDK_JAVA_NATIVE_SAMPLE
    ? resolve(process.env.LITHE_PLUGIN_SDK_JAVA_NATIVE_SAMPLE)
    : join(repository, 'sdk/java/examples/sample-native-build/target/sample-native-build.jar');
  if (buildWorkflows) await access(nativeJar);
  const rust = await rustSample();
  const revision = await checkoutCore(source, ref);
  if (values['working-tree']) await copyWorkingTree(source, work);
  console.log(`Running the SDK end-to-end scenario against Lithe ${revision}${values['working-tree'] ? ' plus explicit working-tree changes' : ''}`);
  await copyScenario(join(work, 'rust/lithe-core/tests'));

  await run('cargo', ['test', '--locked', '--manifest-path', join(work, 'rust/Cargo.toml'), '-p', 'lithe-core',
    ...[...tests, ...(buildWorkflows ? ['plugin_sdk_build'] : [])].flatMap(test => ['--test', test])], {
    env: {
      ...process.env,
      LITHE_PLUGIN_SDK_ROOT: repository,
      LITHE_PLUGIN_SDK_CONFIGURATION_WORKFLOWS: configurationWorkflows ? '1' : '0',
      LITHE_PLUGIN_SDK_NODE: process.execPath,
      LITHE_PLUGIN_SDK_JAVA: java,
      LITHE_PLUGIN_SDK_JAVA_SAMPLE: jar,
      LITHE_PLUGIN_SDK_JAVA_NATIVE_SAMPLE: nativeJar,
      LITHE_PLUGIN_SDK_RUST_SAMPLE: rust,
    },
  });
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
