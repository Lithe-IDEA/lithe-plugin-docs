// Runs the SDK end-to-end scenario in sdk/e2e against Lithe's Rust Core.
//
//   node scripts/sdk-e2e.mjs [--ref <commit|branch|HEAD>] [--source <Lithe git URL or path>]
//
// Core lives in the Lithe repository, so the scenario is copied into
// rust/lithe-core/tests/ of a disposable Lithe checkout in .artifacts/lithe
// (only rust/ and shared/ are checked out) and run there with cargo. By default
// that checkout is the commit contract.lock.json pins, the same Core the
// contract mirror came from; `--ref HEAD` tests Lithe's latest default branch.
//
// The Node.js sample runs on the node running this script. The Java sample
// runs on LITHE_PLUGIN_SDK_JAVA_RUNTIME, or `java` on PATH, and is built with
// Maven unless LITHE_PLUGIN_SDK_JAVA_SAMPLE names a built sample-build.jar.
import { spawn } from 'node:child_process';
import { access, copyFile, mkdir, readdir } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { parseArgs } from 'node:util';
import { readLock, repository } from './contract.mjs';

const work = join(repository, '.artifacts', 'lithe');
const scenario = join(repository, 'sdk', 'e2e');
const tests = ['plugin_sdk_node', 'plugin_sdk_java'];

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

function output(command, args) {
  return new Promise((resolvePromise, reject) => {
    const child = spawn(command, args, { stdio: ['ignore', 'pipe', 'inherit'] });
    let text = '';
    child.stdout.on('data', chunk => { text += chunk; });
    child.on('error', reject);
    child.on('close', code => code === 0
      ? resolvePromise(text.trim())
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
  // Scenario files copied by an earlier run are untracked; drop them so a file
  // removed from sdk/e2e does not linger.
  await run('git', ['-C', work, 'clean', '-q', '-f', '-d', '--', 'rust/lithe-core/tests']);
  return output('git', ['-C', work, 'rev-parse', 'HEAD']);
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

async function main() {
  const { values } = parseArgs({ options: { ref: { type: 'string' }, source: { type: 'string' } } });
  const lock = await readLock();
  const ref = values.ref ?? lock.source.revision;
  const source = values.source ?? `${lock.source.repository}.git`;

  const java = process.env.LITHE_PLUGIN_SDK_JAVA_RUNTIME || await which('java');
  const jar = await javaSample();
  const revision = await checkoutCore(source, ref);
  console.log(`Running the SDK end-to-end scenario against Lithe ${revision}`);
  await copyScenario(join(work, 'rust/lithe-core/tests'));

  await run('cargo', ['test', '--locked', '--manifest-path', join(work, 'rust/Cargo.toml'), '-p', 'lithe-core',
    ...tests.flatMap(test => ['--test', test])], {
    env: {
      ...process.env,
      LITHE_PLUGIN_SDK_ROOT: repository,
      LITHE_PLUGIN_SDK_NODE: process.execPath,
      LITHE_PLUGIN_SDK_JAVA: java,
      LITHE_PLUGIN_SDK_JAVA_SAMPLE: jar,
    },
  });
}

main().catch(error => {
  console.error(error.message);
  process.exitCode = 1;
});
