// Check the host-service declarations the way a plugin uses them.
import { createBackend, HOST_ERRORS, HostError, type ContextReference, type Subscription } from '@lithe/plugin-sdk';

const backend = createBackend();
const version: number | null = backend.functionsVersion;
void version;

backend.function('query.run', async (_params, context) => {
  const secret: string | null = await context.host.credentials.get('connection.prod');
  await context.host.credentials.set('connection.prod', secret ?? '');
  const workspace: ContextReference | null = context.workspaceContext;
  if (!workspace) return null;
  const { folders } = await context.host.workspace.getFolders(workspace);
  const root = folders[0];
  if (!root) return null;
  const file = await context.host.workspace.readFile(workspace, { resource: { rootID: root.rootID, path: 'README.md' }, maximumBytes: 4096 });
  const truncated: boolean = file.isTruncated;
  const { selectedIndex } = await context.host.window.showInformation(workspace, { text: 'Done', actions: ['Open'] });
  const choice: number | null = selectedIndex;
  return { truncated, choice };
});

backend.onCommand('db.refresh', async ({ commandID, workspaceContext }) => {
  const id: string = commandID;
  void id;
  if (!workspaceContext) return;
  if (await backend.host.supports('lithe/host/workspace/watch')) {
    const subscription: Subscription = await backend.host.workspace.watch(
      workspaceContext,
      { resource: { rootID: 'r1', path: '' }, recursive: true },
      (event, { sequence, gap }) => {
        const next: number = sequence;
        const missed: boolean = gap;
        void event; void next; void missed;
      },
    );
    await subscription.dispose();
  }
});

backend.onContextsChanged(async ({ revision }) => {
  const { contexts } = await backend.host.workspace.listContexts();
  const authority: 'local' | 'remote' = contexts[0]!.authority.kind;
  void revision; void authority;
});

async function edit(context: ContextReference) {
  try {
    const result = await backend.host.editor.applyEdits(context, {
      documentID: 'd1',
      expectedVersion: 3,
      edits: [{ range: { start: { line: 1, column: 1 }, end: { line: 1, column: 1 } }, text: 'x' }],
    });
    const applied: boolean = result.applied;
    void applied;
  } catch (error) {
    if (error instanceof HostError && error.rpcCode === HOST_ERRORS['plugin.host.versionConflict']) {
      const details = error.details;
      void details;
    }
  }
  // @ts-expect-error An edit needs the version it was computed against.
  await backend.host.editor.applyEdits(context, { documentID: 'd1', edits: [] });
  // @ts-expect-error A credential secret is a string.
  await backend.host.credentials.set('key', 42);
}
void edit;
