// Type declarations for the Lithe plugin backend SDK. Keep in sync with the
// `.mjs` sources; `sdk/node/test/support.test.mjs` checks the exported names.

/** Any value JSON can carry. */
export type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };
export type JsonObject = { [key: string]: JsonValue };

export interface RunProcessOptions {
  cwd?: string;
  /** Replaces the environment when given; Lithe's process markers are carried over regardless. */
  env?: Record<string, string>;
  /** Aborting it ends the whole process tree and rejects with its reason. */
  signal?: AbortSignal;
  /** Captured bytes per stream; default 1 MiB. */
  maxOutputBytes?: number;
  onStdout?: (chunk: string) => void;
  onStderr?: (chunk: string) => void;
  /** Delay between the polite and forced stop; default 2000. */
  killGraceMilliseconds?: number;
}

export interface ProcessResult {
  exitCode: number | null;
  signal: string | null;
  stdout: string;
  stderr: string;
  truncated: boolean;
}

/** Shared by every handler. */
export interface RequestContext {
  /** Aborted when the host cancels the call, the call times out, the session ends, or the call has settled. */
  signal: AbortSignal;
  throwIfCancelled(): void;
  /** `runProcess` bound to this call: its tree ends when the call is cancelled or settles. */
  run(command: string, args?: string[], options?: RunProcessOptions): Promise<ProcessResult>;
  /** Writes a line to stderr, which the host keeps in its log. */
  log(...parts: unknown[]): void;
}

export interface ProgressReport {
  /** Cut to 1024 bytes. */
  message?: string;
  /** Rounded and clamped to 0-100. */
  percentage?: number;
}

export interface FunctionContext extends RequestContext {
  functionID: string;
  /** Opaque per-call ID chosen by Lithe. */
  operationID: string;
  /** Reports progress; ignored once the call has ended. */
  progress(report: ProgressReport): void;
}

export type FunctionHandler = (params: JsonObject, context: FunctionContext) => unknown | Promise<unknown>;
export type BuildHandler = (params: BuildRequestParams, context: RequestContext) => JsonObject | Promise<JsonObject>;

/** Params of `lithe/build/model` and `lithe/build/dependencies`. */
export interface BuildRequestParams {
  /** Read-only host snapshot; missing producers remain unavailable. Requires protocol 6. */
  modelInputs?: BuildModelInputs;
  buildSystemID: string;
  root: string;
  options: JsonObject;
  optionArguments: string[];
  environment: Record<string, string>;
  /** Dependencies only: the selected project, or null for the workspace. */
  project?: { name: string; path: string } | null;
  /** True when the user or a sync-file change asked to reload; skip any snapshot. */
  refresh?: boolean;
  /** The build tool the host resolved; absent when it could not, so look it up yourself. */
  tool?: { executable: string };
  [key: string]: unknown;
}

/** Public normalized facts. Tasks, run plans, settings and optionValues never cross this boundary. */
export interface BuildSharedModel {
  projects: { id: string; name: string; path: string; displayName?: string; parentID?: string; properties: Record<string, string> }[];
  targets: { id: string; projectID: string; name: string; path: string; tags: string[]; properties: Record<string, string> }[];
  contentRoots: { projectID: string; path: string; kind: string }[];
}
export type BuildModelInputInstance =
  | { root: string; state: 'ready'; model: BuildSharedModel; configurationFingerprint: string; options: JsonObject }
  | { root: string; state: 'loading' | 'stale' | 'failed' };
export interface BuildModelInputs {
  version: 1;
  /** Opaque workspace-activation revision; discard cached work after it changes. */
  revision: number;
  inputs: { pluginID: string; buildSystemID: string; state: 'available' | 'unavailable'; instances: BuildModelInputInstance[] }[];
}

/** Current upstream model selection, preserved as data rather than shell text. */
export interface BuildSelection {
  name: string;
  path: string;
  tags?: string[];
  properties?: JsonObject;
}

export interface BuildExecutionParams extends BuildRequestParams {
  task: string;
  project?: BuildSelection | null;
  target?: BuildSelection | null;
  userArguments: string[];
  operationID: string;
}

export interface BuildTestParams extends BuildRequestParams {
  /** Protocol 9: public semantic namespace and upstream-owned opaque identities. */
  selectorKind?: string;
  selectors?: string[];
  /** Requires build protocol 8 and an inline report in the reply. */
  testResults?: true;
  project?: BuildSelection | null;
  target?: BuildSelection | null;
  suite?: string;
  case?: string;
  userArguments: string[];
  operationID: string;
}

/** Version 10. These are resolved user choices, not shell command strings. */
export interface BuildLaunchOptions {
  execution: 'application' | 'service' | 'task';
  runtimeArguments: string[];
  arguments: string[];
  environment: Record<string, string>;
  workingDirectory?: string;
  entrypoint?: string;
}

export interface BuildRunParams extends BuildRequestParams {
  launchOptions?: BuildLaunchOptions;
  run: string;
  project: BuildSelection;
  target: BuildSelection;
  userArguments: string[];
  operationID: string;
}

export interface BuildProgressReport extends ProgressReport {
  /** Cut to 64 KiB of UTF-8 per notification. Build messages are cut to 4096 bytes. */
  output?: string;
}

export interface BuildContext extends RequestContext {
  /** The Core-assigned wire identity, never a plugin-selected value. */
  operationID: string;
  /** Dropped after cancellation or completion. */
  progress(report: BuildProgressReport): void;
}

/** Results belong to this operation; the host does not discover report files. */
export interface BuildTestResultsDocument {
  format: 'litheTestResults'; version: 1;
  /** Completed project/suite/case scopes; omitted means only returned cases are replaced. */
  coverage?: { project: string; suite?: string; case?: string }[];
  tests: { project?: string; suite: string; case: string; status: 'passed' | 'failed' | 'error' | 'skipped';
    message?: string; durationMillis?: number; path?: string; line?: number; invocations?: number }[];
}
export interface BuildExecutionResult { exitCode: number; output: string; testResults?: BuildTestResultsDocument; }
export interface BuildTestDiscovery {
  format?: 'litheTestDiscovery';
  version?: 1;
  tests: { suite: string; case: string; project?: BuildSelection; target?: BuildSelection }[];
}
export interface BuildRunPreparation {
  executable: { kind: 'toolchain' | 'command'; value: string };
  arguments: string[];
  workingDirectory: string;
  environment: Record<string, string>;
}
export type BuildExecutionHandler = (params: BuildExecutionParams, context: BuildContext) => BuildExecutionResult | Promise<BuildExecutionResult>;
export type BuildTestHandler = (params: BuildTestParams, context: BuildContext) => BuildExecutionResult | Promise<BuildExecutionResult>;
export type BuildDiscoveryHandler = (params: BuildRequestParams, context: RequestContext) => BuildTestDiscovery | Promise<BuildTestDiscovery>;
export interface BuildDebugPreparation {
  program: string;
  arguments: string[];
  workingDirectory: string;
  environment: Record<string, string>;
}
export type BuildDebugHandler = (params: BuildRunParams, context: BuildContext) => BuildDebugPreparation | Promise<BuildDebugPreparation>;
export type BuildRunHandler = (params: BuildRunParams, context: BuildContext) => BuildRunPreparation | Promise<BuildRunPreparation>;

/** Params every VCS handler receives, on top of its operation's arguments. */
export interface VcsRequestParams {
  root: { path: string };
  /** The VCS client the host resolved; null when it could not, so report `vcs.tool.unavailable`. */
  tool: { executable: string } | null;
  [key: string]: unknown;
}

export interface VcsRunOptions extends RunProcessOptions {
  /** Report the finished command as `lithe/vcs/execution`; default true. */
  report?: boolean;
}

export interface VcsContext extends RequestContext {
  /** `RequestContext.run` that also reports each finished command to the host's console. */
  run(command: string, args?: string[], options?: VcsRunOptions): Promise<ProcessResult>;
}

export type VcsHandler = (params: VcsRequestParams, context: VcsContext) => JsonObject | void | Promise<JsonObject | void>;

/** Handlers keyed by operation; see shared/contracts/vcs-provider-v1.md for each reply. */
export interface VcsHandlers {
  status: VcsHandler;
  baseContent: VcsHandler;
  checkin: VcsHandler;
  rollback: VcsHandler;
  history?: VcsHandler;
  fileHistory?: VcsHandler;
  annotate?: VcsHandler;
  update?: VcsHandler;
  fileOperation?: VcsHandler;
  resolve?: VcsHandler;
  cleanup?: VcsHandler;
}

export interface VcsTraits {
  /** Check-in is local and publishing is a separate step. */
  push?: boolean;
  /** Changes carry a staged/unstaged dimension. */
  staging?: boolean;
}

export interface Negotiated {
  functions: boolean;
  build: boolean;
  vcs: boolean;
}

export interface BackendOptions {
  name?: string;
  version?: string;
  /** Standard LSP server capabilities; `experimental.litheBuild`, `litheFunctions`, and `litheVcs` are reserved. */
  capabilities?: JsonObject;
}

export interface ListenOptions {
  input?: NodeJS.ReadableStream;
  output?: NodeJS.WritableStream;
  errorOutput?: NodeJS.WritableStream;
  /** Defaults to `process.exit`; null only resolves the returned promise. */
  exit?: ((code: number) => void) | null;
}

export interface Backend {
  readonly negotiated: Negotiated;
  readonly initializeParams: JsonObject | null;
  /** Highest common version (1-8), or null before/without negotiation. */
  readonly buildVersion: number | null;
  readonly functionIDs: string[];
  readonly build: {
    /** Non-sensitive source-backed settings; requires build protocol 7. */
    configuration(handler: BuildHandler): Backend;
    model(handler: BuildHandler): Backend;
    dependencies(handler: BuildHandler): Backend;
    execute(handler: BuildExecutionHandler): Backend;
    test(handler: BuildTestHandler): Backend;
    discoverTests(handler: BuildDiscoveryHandler): Backend;
    prepareRun(handler: BuildRunHandler): Backend;
    prepareDebug(handler: BuildDebugHandler): Backend;
  };
  vcs(id: string, handlers: VcsHandlers, traits?: VcsTraits): Backend;
  function(id: string, handler: FunctionHandler): Backend;
  registerFunction(id: string, handler: FunctionHandler): Promise<void>;
  unregisterFunction(id: string): Promise<void>;
  onRequest(method: string, handler: (params: unknown, context: RequestContext) => unknown): Backend;
  onNotification(method: string, handler: (params: unknown) => unknown): Backend;
  onInitialize(hook: (event: { params: JsonObject; negotiated: Negotiated }) => unknown): Backend;
  onShutdown(hook: () => unknown): Backend;
  sendRequest(method: string, params?: unknown): Promise<unknown>;
  sendNotification(method: string, params?: unknown): void;
  log(...parts: unknown[]): void;
  listen(options?: ListenOptions): Promise<number>;
}

export function createBackend(options?: BackendOptions): Backend;

export class PluginError extends Error {
  constructor(code: string, message: string, options?: { cause?: unknown });
  readonly code: string;
}

export class PluginSdkError extends Error {
  readonly code: string;
}

export function runProcess(command: string, args?: string[], options?: RunProcessOptions): Promise<ProcessResult>;

export interface StateStore {
  readonly directory: string;
  read<T = JsonValue>(key: string, fallback?: T): T | undefined;
  write(key: string, value: JsonValue): void;
  delete(key: string): void;
}

export function openStateStore(directory: string, options?: { packageDirectory?: string | null }): StateStore;

export function isFunctionId(value: unknown): boolean;

export function isVcsId(value: unknown): boolean;

/** Provider error codes Core passes on; use them as `PluginError` codes. */
export const VCS_ERRORS: Readonly<{
  authenticationFailed: 'vcs.authentication.failed';
  conflict: 'vcs.conflict';
  outOfDate: 'vcs.outOfDate';
  toolUnavailable: 'vcs.tool.unavailable';
  workingCopyLocked: 'vcs.workingCopy.locked';
  workingCopyUnsupported: 'vcs.workingCopy.unsupported';
}>;

export const LIMITS: Readonly<{
  maxFunctions: number;
  maxFunctionIdBytes: number;
  maxPluginErrorCodeBytes: number;
  maxMessageBytes: number;
  maxResultBytes: number;
  maxBuildResultBytes: number;
  maxBuildMessageBytes: number;
  maxBuildOutputBytes: number;
  maxVcsIdBytes: number;
  maxExecutionErrorOutputBytes: number;
}>;
