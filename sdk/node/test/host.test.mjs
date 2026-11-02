import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { createBackend, HOST_ERRORS, HostError, LIMITS, PluginSdkError } from '../src/index.mjs';
import { gate, TestHost, withTimeout } from './harness.mjs';

const fixture = JSON.parse(
  readFileSync(new URL('../../../contract/v1/fixtures/function-protocol.json', import.meta.url), 'utf8'),
);

const CONTEXT = Object.freeze({ contextID: 'w1', generation: 2 });

/** A backend and host whose session negotiated plugin-function `version`. */
async function session(version, backend = createBackend()) {
  const host = new TestHost(backend);
  const versions = Array.from({ length: version }, (_, index) => index + 1);
  await host.initialize({ litheFunctions: { versions } });
  return { backend, host };
}

/** The next host request the backend wrote that the test has not taken yet. */
function nextRequest(host, method, taken = new Set()) {
  return host
    .waitFor((message) => message.method === method && message.id !== undefined && !taken.has(message.id), method)
    .then((message) => {
      taken.add(message.id);
      return message;
    });
}

test('the SDK answers the highest plugin-function version the host offers', async () => {
  for (const [offered, expected] of [[fixture.offer.versions, 3], [[1, 2], 2], [[1], 1], [[2, 99], 2]]) {
    const backend = createBackend().function('query.run', () => null);
    const host = new TestHost(backend);
    const { result } = await host.initialize({ litheFunctions: { versions: offered } });
    assert.deepEqual(result.capabilities.experimental.litheFunctions, { version: expected, functions: [{ id: 'query.run' }] });
    assert.equal(backend.functionsVersion, expected);
    assert.equal(backend.host.version, expected);
    await host.close();
  }
  // The fixture's answers of versions 2 and 3 have the shape the SDK sends.
  for (const testCase of fixture.negotiation.filter(({ expected }) => expected.version >= 2)) {
    assert.deepEqual(Object.keys(testCase.capabilities.experimental.litheFunctions).sort(), ['functions', 'version'], testCase.name);
  }
});

test('credentials round-trip through the host, with the contract shapes', async () => {
  const { backend, host } = await session(2);
  const taken = new Set();

  const reading = backend.host.credentials.get('connection.prod');
  const get = await nextRequest(host, 'lithe/host/credentials/get', taken);
  assert.deepEqual(get.params, { key: 'connection.prod' });
  host.send({ id: get.id, result: { secret: 's3cret' } });
  assert.equal(await reading, 's3cret');

  const missing = backend.host.credentials.get('connection.prod');
  host.send({ id: (await nextRequest(host, 'lithe/host/credentials/get', taken)).id, result: null });
  assert.equal(await missing, null);

  const malformed = backend.host.credentials.get('connection.prod');
  host.send({ id: (await nextRequest(host, 'lithe/host/credentials/get', taken)).id, result: {} });
  await assert.rejects(malformed, (error) => error.code === 'sdk.host.invalidResult');

  const writing = backend.host.credentials.set('connection.prod:password', 's3cret');
  const set = await nextRequest(host, 'lithe/host/credentials/set', taken);
  assert.deepEqual(set.params, { key: 'connection.prod:password', secret: 's3cret' });
  host.send({ id: set.id, result: null });
  assert.equal(await writing, undefined);

  const deleting = backend.host.credentials.delete('connection.prod');
  const remove = await nextRequest(host, 'lithe/host/credentials/delete', taken);
  assert.deepEqual(remove.params, { key: 'connection.prod' });
  host.send({ id: remove.id, result: null });
  await deleting;

  // Malformed arguments fail locally and never reach the host.
  const before = host.messages.length;
  await assert.rejects(backend.host.credentials.get(''), (error) => error.code === 'sdk.credentials.invalidKey');
  await assert.rejects(backend.host.credentials.get('has space'), (error) => error.code === 'sdk.credentials.invalidKey');
  await assert.rejects(backend.host.credentials.get('k'.repeat(129)), (error) => error.code === 'sdk.credentials.invalidKey');
  await assert.rejects(backend.host.credentials.set('key', 7), (error) => error.code === 'sdk.credentials.invalidSecret');
  await assert.rejects(
    backend.host.credentials.set('key', 'x'.repeat(LIMITS.maxCredentialSecretBytes + 1)),
    (error) => error.code === 'sdk.credentials.invalidSecret',
  );
  assert.equal(host.messages.length, before);
  await host.close();
});

test('local key and context checks agree with the fixture cases Core refuses or forwards', async () => {
  const { backend, host } = await session(3);
  const key = (testCase) => testCase.params?.key;
  for (const testCase of fixture.hostRequests.filter((testCase) => testCase.method.startsWith('lithe/host/credentials/'))) {
    if (testCase.expected.forward) {
      assert.match(key(testCase), /^[A-Za-z0-9._:-]{1,128}$/, testCase.name);
    } else if (testCase.name.includes('key')) {
      await assert.rejects(backend.host.credentials.get(key(testCase)), (error) => error.code === 'sdk.credentials.invalidKey', testCase.name);
    }
  }
  for (const testCase of fixture.hostRequests.filter((testCase) => /context (without|ID outside)|generation is positive/.test(testCase.name))) {
    await assert.rejects(
      backend.host.workspace.getFolders(testCase.params.context),
      (error) => error.code === 'sdk.host.invalidContext',
      testCase.name,
    );
  }
  // A full context from listContexts is passed on as its reference.
  const asking = backend.host.workspace.getFolders({ ...CONTEXT, name: 'orders', folders: [] });
  const request = await nextRequest(host, 'lithe/host/workspace/getFolders');
  assert.deepEqual(request.params, { context: CONTEXT });
  host.send({ id: request.id, result: { folders: [] } });
  assert.deepEqual(await asking, { folders: [] });
  await host.close();
});

test('host refusals become HostError with the code, number, and details Core sent', async () => {
  const { backend, host } = await session(3);
  const taken = new Set();
  const answer = async (wire) => {
    const pending = backend.host.request('lithe/host/workspace/listContexts');
    const request = await nextRequest(host, 'lithe/host/workspace/listContexts', taken);
    host.send({ id: request.id, error: wire });
    return pending.then(() => assert.fail('resolved'), (error) => error);
  };
  for (const testCase of fixture.hostRequests.filter(({ expected }) => expected.code)) {
    const { code, rpcCode, details } = testCase.expected;
    const error = await answer({ code: rpcCode, message: 'Refused.', data: { code, ...(details ? { details } : {}) } });
    assert.ok(error instanceof HostError && error instanceof PluginSdkError, testCase.name);
    assert.deepEqual({ code: error.code, rpcCode: error.rpcCode, details: error.details }, { code, rpcCode, details }, testCase.name);
    assert.equal(error.method, 'lithe/host/workspace/listContexts');
  }
  for (const testCase of fixture.hostFailures) {
    const { code, rpcCode } = testCase.expected;
    assert.equal(HOST_ERRORS[code], rpcCode, testCase.name);
    // The number alone names the code.
    assert.equal((await answer({ code: rpcCode, message: 'Failed.' })).code, code, testCase.name);
  }
  const unknown = await answer({ code: -32050, message: 'Later code.', data: { details: ['not', 'an', 'object'] } });
  assert.deepEqual({ code: unknown.code, details: unknown.details }, { code: 'plugin.host.failed', details: undefined });
  await host.close();
});

test('protocol 1 offers no host service, and host requests stay within lithe/host/', async () => {
  const { backend, host } = await session(1);
  const before = host.messages.length;
  await assert.rejects(
    backend.host.credentials.get('connection.prod'),
    (error) => error instanceof HostError && error.code === 'plugin.host.unavailable' && error.rpcCode === -32601,
  );
  await assert.rejects(backend.host.capabilities(), (error) => error.code === 'plugin.host.unavailable');
  assert.equal(await backend.host.supports('lithe/host/credentials/get'), false);
  await assert.rejects(backend.host.request('lithe/function/invoke', {}), (error) => error.code === 'sdk.host.method');
  assert.equal(host.messages.length, before);
  await host.close();

  const idle = createBackend();
  await assert.rejects(idle.host.credentials.get('key'), (error) => error.code === 'sdk.session.notReady');
});

test('capabilities are asked for once, and supports() reads them', async () => {
  const { backend, host } = await session(3);
  const answer = fixture.hostRequests.find((testCase) => testCase.expected.answer).expected.answer;
  const first = backend.host.supports('lithe/host/workspace/getFolders');
  const second = backend.host.supports('lithe/host/workspace/readFile');
  const request = await nextRequest(host, 'lithe/host/getCapabilities');
  assert.equal(request.params, undefined);
  host.send({ id: request.id, result: answer });
  assert.deepEqual([await first, await second], [true, false]);
  assert.equal((await backend.host.capabilities()).limits.pendingHostRequests, LIMITS.maxPendingHostRequests);
  assert.equal(host.messages.filter((message) => message.method === 'lithe/host/getCapabilities').length, 1);
  await host.close();

  // Version 2 has no discovery: it reports what version 2 defines.
  const older = await session(2);
  assert.equal(await older.backend.host.supports('lithe/host/credentials/set'), true);
  assert.equal(await older.backend.host.supports('lithe/host/workspace/listContexts'), false);
  await older.host.close();
});

test('a version 3 call carries its workspace context to the handler; older ones do not', async () => {
  for (const testCase of fixture.invocationContexts.filter(({ expected }) => expected.params)) {
    let seen;
    const backend = createBackend().function(testCase.params.functionID, (_params, context) => {
      seen = context.workspaceContext;
    });
    const { host } = await session(testCase.version, backend);
    const response = await host.request('lithe/function/invoke', testCase.expected.params);
    assert.equal(response.error, undefined, testCase.name);
    assert.deepEqual(seen, testCase.expected.params.context ?? null, testCase.name);
    await host.close();
  }
});

test('cancelling a call cancels its host requests: version 3 tells Core, version 2 drops the answer', async () => {
  for (const version of [3, 2]) {
    const started = gate();
    let outcome;
    const backend = createBackend().function('secret.read', async (_params, context) => {
      const reading = context.host.credentials.get('connection.prod');
      started.open();
      outcome = await reading.then(() => 'resolved', (error) => error);
      throw outcome;
    });
    const { host } = await session(version, backend);
    const call = host.request('lithe/function/invoke', { functionID: 'secret.read', operationID: 'c' }, 41);
    await withTimeout(started.opened, 'the host request');
    const request = await nextRequest(host, 'lithe/host/credentials/get');
    host.notify('$/cancelRequest', { id: 41 });
    assert.equal((await call).error.code, -32800, `version ${version}`);
    // The host request rejects with the call's abort reason.
    assert.equal(outcome?.name, 'AbortError');
    const cancels = host.messages.filter((message) => message.method === '$/cancelRequest');
    assert.deepEqual(cancels.map((message) => message.params), version === 3 ? [{ id: request.id }] : [], `version ${version}`);
    // Core's late answer is dropped; the session continues.
    host.send({ id: request.id, error: { code: -32800, message: 'Cancelled.', data: { code: 'plugin.host.cancelled' } } });
    assert.equal(await host.close(), 1);
  }
});

test('an aborted signal rejects with its reason and is never sent again', async () => {
  const { backend, host } = await session(3);
  const controller = new AbortController();
  const pending = backend.host.editor.applyEdits(CONTEXT, { documentID: 'd1', expectedVersion: 4, edits: [] }, { signal: controller.signal });
  const request = await nextRequest(host, 'lithe/host/editor/applyEdits');
  assert.deepEqual(request.params, { documentID: 'd1', expectedVersion: 4, edits: [], context: CONTEXT });
  controller.abort(new Error('user stopped'));
  await assert.rejects(pending, /user stopped/);
  host.send({ id: request.id, result: { documentID: 'd1', version: 5, applied: true } });
  await assert.rejects(backend.host.request('lithe/host/editor/applyEdits', {}, { signal: controller.signal }), /user stopped/);
  assert.equal(host.messages.filter((message) => message.method === 'lithe/host/editor/applyEdits').length, 1);
  await host.close();
});

test('at most 16 host requests are unanswered; later ones wait for a free slot', async () => {
  const { backend, host } = await session(3);
  const max = LIMITS.maxPendingHostRequests;
  const calls = Array.from({ length: max + 2 }, () => backend.host.workspace.listContexts());
  const sent = () => host.messages.filter((message) => message.method === 'lithe/host/workspace/listContexts');
  await host.waitFor(() => sent().length === max, 'the first 16 requests');
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(sent().length, max);
  host.send({ id: sent()[0].id, result: { revision: 1, contexts: [] } });
  await host.waitFor(() => sent().length === max + 1, 'the 17th request');
  assert.equal(sent().length, max + 1);
  // Ending the session fails the queued request too.
  await host.close();
  const outcomes = await Promise.allSettled(calls);
  assert.equal(outcomes[0].status, 'fulfilled');
  assert.ok(outcomes.slice(1).every((outcome) => outcome.status === 'rejected' && outcome.reason.code === 'sdk.session.closed'));
});

test('subscription events reach their handler in order, also when sent before the answer', async () => {
  const { backend, host } = await session(3);
  const events = [];
  const received = gate();
  const watching = backend.host.workspace.watch(CONTEXT, { resource: { rootID: 'r1', path: '' }, recursive: true }, (event, info) => {
    events.push({ event, ...info });
    if (events.length === 3) received.open();
  });
  const request = await nextRequest(host, 'lithe/host/workspace/watch');
  assert.deepEqual(request.params, { resource: { rootID: 'r1', path: '' }, recursive: true, context: CONTEXT });
  const notify = (sequence) => host.notify('lithe/host/subscriptions/notify', {
    subscriptionID: 's1', sequence, event: { rootID: 'r1', changedPaths: [`f${sequence}`], requiresRescan: false },
  });
  notify(1);
  host.send({ id: request.id, result: { subscriptionID: 's1' } });
  const subscription = await watching;
  notify(2);
  notify(4);
  await withTimeout(received.opened, 'three events');
  assert.deepEqual(events.map(({ sequence, gap, subscriptionID }) => ({ sequence, gap, subscriptionID })), [
    { sequence: 1, gap: false, subscriptionID: 's1' },
    { sequence: 2, gap: false, subscriptionID: 's1' },
    { sequence: 4, gap: true, subscriptionID: 's1' },
  ]);
  assert.equal(subscription.subscriptionID, 's1');
  assert.equal(subscription.disposed, false);

  const disposing = subscription.dispose();
  assert.equal(subscription.disposed, true);
  const dispose = await nextRequest(host, 'lithe/host/subscriptions/dispose');
  assert.deepEqual(dispose.params, { subscriptionID: 's1' });
  notify(5);
  host.send({ id: dispose.id, result: null });
  await disposing;
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(events.length, 3);
  await host.close();
});

test('commands, context changes, and later host notifications reach their handlers', async () => {
  const delivered = fixture.hostNotifications.filter(({ expected, method }) => expected.delivered && method !== 'lithe/host/subscriptions/notify');
  const expected = [
    ...delivered.map(({ method, params }) => (method === 'lithe/host/commands/didRun'
      ? ['command', { commandID: params.commandID, workspaceContext: params.context ?? null }]
      : ['contexts', params])),
    ['later', { value: 1 }],
  ];
  const backend = createBackend();
  const seen = [];
  const all = gate();
  const record = (entry) => {
    seen.push(entry);
    if (seen.length === expected.length) all.open();
  };
  backend
    .onCommand('db.refresh', (event) => record(['command', event]))
    .onContextsChanged((event) => record(['contexts', event]))
    .onHostNotification('lithe/host/later/didSomething', (params) => record(['later', params]));
  assert.throws(() => backend.onHostNotification('lithe/host/commands/didRun', () => {}), (error) => error.code === 'sdk.method.reserved');
  assert.throws(() => backend.onHostNotification('workspace/didChange', () => {}), (error) => error.code === 'sdk.method.invalid');
  assert.throws(() => backend.onCommand('DB_Refresh', () => {}), (error) => error.code === 'sdk.command.invalidID');
  const { host } = await session(3, backend);
  for (const { method, params } of delivered) host.notify(method, params);
  // A notification this SDK does not know is ignored, as the contract asks.
  host.notify('lithe/host/unknown/ignored', { anything: true });
  host.notify('lithe/host/later/didSomething', { value: 1 });
  await withTimeout(all.opened, 'the notifications');
  assert.deepEqual(seen, expected);
  await host.close();
});

test('native terminal wrappers keep argv and context, and subscribe to one owned exit', { timeout: 5000 }, async () => {
  const { backend, host } = await session(3);
  try {
    const params = { executable: 'example-client', arguments: ['literal;$(value)', 'space value'], show: true };
    const creating = backend.host.terminal.create(CONTEXT, params);
    const create = await nextRequest(host, 'lithe/host/terminal/create');
    assert.deepEqual(create.params, { ...params, context: CONTEXT });
    host.send({ id: create.id, result: { terminalID: 'owned', generation: 1 } });
    assert.deepEqual(await creating, { terminalID: 'owned', generation: 1 });
    const seen = gate();
    let event;
    const listening = backend.host.terminal.onExit(CONTEXT, 'owned', value => { event = value; seen.open(); });
    const listen = await nextRequest(host, 'lithe/host/terminal/onExit');
    host.send({ id: listen.id, result: { subscriptionID: 'terminal-exit' } });
    const subscription = await listening;
    host.notify('lithe/host/subscriptions/notify', { subscriptionID: 'terminal-exit', sequence: 1,
      event: { type: 'exit', terminalID: 'owned', exitCode: 0, reason: 'exited' } });
    await withTimeout(seen.opened, 'terminal exit event');
    assert.equal(event.exitCode, 0);
    const closing = backend.host.terminal.close(CONTEXT, { terminalID: 'owned' });
    const close = await nextRequest(host, 'lithe/host/terminal/close');
    host.send({ id: close.id, result: { retainedContextBytes: 0 } });
    assert.deepEqual(await closing, { retainedContextBytes: 0 });
    const reopening = backend.host.terminal.reopen(CONTEXT, { terminalID: 'owned' });
    const reopen = await nextRequest(host, 'lithe/host/terminal/reopen');
    host.send({ id: reopen.id, result: { terminalID: 'owned', generation: 2, restoredContextBytes: 0 } });
    assert.equal((await reopening).generation, 2);
    const disposing = subscription.dispose();
    const dispose = await nextRequest(host, 'lithe/host/subscriptions/dispose');
    host.send({ id: dispose.id, result: null });
    await disposing;
  } finally { await host.close(); }
});
