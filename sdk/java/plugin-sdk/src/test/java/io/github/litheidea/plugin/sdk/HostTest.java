package io.github.litheidea.plugin.sdk;

import static io.github.litheidea.plugin.sdk.Fixtures.map;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;
import java.util.stream.Collectors;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;

/** Host services of plugin-function protocols 2 and 3; mirrors {@code sdk/node/test/host.test.mjs}. */
@Timeout(value = 30, unit = TimeUnit.SECONDS)
class HostTest {
    private static final Map<String, Object> CONTEXT = Collections.unmodifiableMap(map("contextID", "w1", "generation", 2L));
    private static final Map<String, Object> FIXTURE = loadFixture();

    private static Map<String, Object> loadFixture() {
        try {
            return object(Json.parse(new String(Files.readAllBytes(
                    Paths.get("../../../contract/v1/fixtures/function-protocol.json")), StandardCharsets.UTF_8)));
        } catch (IOException error) {
            throw new UncheckedIOException(error);
        }
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> object(Object value) {
        return (Map<String, Object>) value;
    }

    private static List<Map<String, Object>> cases(String key) {
        List<Map<String, Object>> cases = new ArrayList<>();
        for (Object entry : (List<?>) FIXTURE.get(key)) {
            cases.add(object(entry));
        }
        return cases;
    }

    /** A host whose session negotiated plugin-function {@code version}. */
    private static TestHost session(int version, Backend backend) {
        TestHost host = new TestHost(backend);
        List<Object> versions = new ArrayList<>();
        for (long offered = 1; offered <= version; offered++) {
            versions.add(offered);
        }
        host.initialize(map("litheFunctions", map("versions", versions)));
        return host;
    }

    /** The next host request the backend wrote that the test has not taken yet. */
    private static Map<String, Object> nextRequest(TestHost host, String method, Set<Object> taken) {
        Map<String, Object> request = host.waitFor(message -> method.equals(message.get("method"))
                && message.get("id") != null && !taken.contains(message.get("id")), method);
        taken.add(request.get("id"));
        return request;
    }

    private static Map<String, Object> nextRequest(TestHost host, String method) {
        return nextRequest(host, method, new HashSet<>());
    }

    private static <T> T value(CompletableFuture<T> future) {
        return Fixtures.await(future, "the host request");
    }

    /** What {@code future} failed with. */
    private static Throwable failure(CompletableFuture<?> future) {
        try {
            future.get(TestHost.WAIT_MILLIS, TimeUnit.MILLISECONDS);
        } catch (ExecutionException error) {
            return error.getCause();
        } catch (InterruptedException | TimeoutException error) {
            throw new AssertionError("The host request did not fail in time.", error);
        }
        return fail("The host request succeeded.");
    }

    private static String code(CompletableFuture<?> future) {
        Throwable error = failure(future);
        assertTrue(error instanceof PluginSdkError, String.valueOf(error));
        return ((PluginSdkError) error).code();
    }

    private static List<Map<String, Object>> sent(TestHost host, String method) {
        return host.messages().stream().filter(message -> method.equals(message.get("method"))).collect(Collectors.toList());
    }

    @Test
    void theSdkAnswersTheHighestPluginFunctionVersionTheHostOffers() {
        Object offer = object(FIXTURE.get("offer")).get("versions");
        Object[][] cases = {{offer, 3}, {Arrays.asList(1L, 2L), 2}, {Arrays.asList(1L), 1}, {Arrays.asList(2L, 99L), 2}};
        for (Object[] testCase : cases) {
            Backend backend = Backend.create(null, null).function("query.run", (params, context) -> null);
            TestHost host = new TestHost(backend);
            Map<String, Object> result = object(host.initialize(map("litheFunctions", map("versions", testCase[0]))).get("result"));
            int expected = (Integer) testCase[1];
            assertEquals(map("version", (long) expected, "functions", Arrays.asList(map("id", "query.run"))),
                    object(object(result.get("capabilities")).get("experimental")).get("litheFunctions"));
            assertEquals(expected, backend.negotiated().functionsVersion());
            assertEquals(expected, backend.host().version());
            host.close();
        }
        // The fixture's answers of versions 2 and 3 have the shape the SDK sends.
        for (Map<String, Object> testCase : cases("negotiation")) {
            Object version = object(testCase.get("expected")).get("version");
            if (version instanceof Number && ((Number) version).intValue() >= 2) {
                Map<String, Object> answer = object(object(object(testCase.get("capabilities")).get("experimental")).get("litheFunctions"));
                assertEquals(new HashSet<>(Arrays.asList("functions", "version")), answer.keySet(), (String) testCase.get("name"));
            }
        }
    }

    @Test
    void credentialsRoundTripThroughTheHostWithTheContractShapes() {
        Backend backend = Backend.create(null, null);
        TestHost host = session(2, backend);
        Host.Credentials credentials = backend.host().credentials();
        Set<Object> taken = new HashSet<>();

        CompletableFuture<String> reading = credentials.get("connection.prod");
        Map<String, Object> get = nextRequest(host, Protocol.HOST_CREDENTIALS_GET, taken);
        assertEquals(map("key", "connection.prod"), get.get("params"));
        host.send(map("id", get.get("id"), "result", map("secret", "s3cret")));
        assertEquals("s3cret", value(reading));

        CompletableFuture<String> missing = credentials.get("connection.prod");
        host.send(map("id", nextRequest(host, Protocol.HOST_CREDENTIALS_GET, taken).get("id"), "result", null));
        assertNull(value(missing));

        CompletableFuture<String> malformed = credentials.get("connection.prod");
        host.send(map("id", nextRequest(host, Protocol.HOST_CREDENTIALS_GET, taken).get("id"), "result", map()));
        assertEquals("sdk.host.invalidResult", code(malformed));

        CompletableFuture<Void> writing = credentials.set("connection.prod:password", "s3cret");
        Map<String, Object> set = nextRequest(host, Protocol.HOST_CREDENTIALS_SET, taken);
        assertEquals(map("key", "connection.prod:password", "secret", "s3cret"), set.get("params"));
        host.send(map("id", set.get("id"), "result", null));
        assertNull(value(writing));

        CompletableFuture<Void> deleting = credentials.delete("connection.prod");
        Map<String, Object> remove = nextRequest(host, Protocol.HOST_CREDENTIALS_DELETE, taken);
        assertEquals(map("key", "connection.prod"), remove.get("params"));
        host.send(map("id", remove.get("id"), "result", null));
        value(deleting);

        // Malformed arguments fail locally and never reach the host.
        int before = host.messages().size();
        StringBuilder longKey = new StringBuilder();
        for (int index = 0; index < 129; index++) {
            longKey.append('k');
        }
        for (String key : Arrays.asList("", "has space", longKey.toString(), null)) {
            assertEquals("sdk.credentials.invalidKey", code(credentials.get(key)), String.valueOf(key));
        }
        StringBuilder secret = new StringBuilder();
        for (int index = 0; index <= Protocol.MAX_CREDENTIAL_SECRET_BYTES; index++) {
            secret.append('x');
        }
        assertEquals("sdk.credentials.invalidSecret", code(credentials.set("key", null)));
        assertEquals("sdk.credentials.invalidSecret", code(credentials.set("key", secret.toString())));
        assertEquals(before, host.messages().size());
        host.close();
    }

    @Test
    void localKeyAndContextChecksAgreeWithTheFixtureCasesCoreRefusesOrForwards() {
        Backend backend = Backend.create(null, null);
        TestHost host = session(3, backend);
        for (Map<String, Object> testCase : cases("hostRequests")) {
            String name = (String) testCase.get("name");
            String method = (String) testCase.get("method");
            Map<String, Object> params = object(testCase.get("params"));
            Map<String, Object> expected = object(testCase.get("expected"));
            if (method.startsWith("lithe/host/credentials/")) {
                String key = (String) params.get("key");
                if (expected.get("forward") != null) {
                    assertTrue(Protocol.isCredentialKey(key), name);
                } else if (name.contains("key")) {
                    assertEquals("sdk.credentials.invalidKey", code(backend.host().credentials().get(key)), name);
                }
            }
            if (name.matches(".*(context (without|ID outside)|generation is positive).*")) {
                assertEquals("sdk.host.invalidContext",
                        code(backend.host().workspace().getFolders(object(params.get("context")))), name);
            }
        }
        // A full context from listContexts is passed on as its reference.
        Map<String, Object> full = map("contextID", "w1", "generation", 2L, "name", "orders", "folders", Collections.emptyList());
        CompletableFuture<Map<String, Object>> asking = backend.host().workspace().getFolders(full);
        Map<String, Object> request = nextRequest(host, Protocol.HOST_WORKSPACE_GET_FOLDERS);
        assertEquals(map("context", CONTEXT), request.get("params"));
        host.send(map("id", request.get("id"), "result", map("folders", Collections.emptyList())));
        assertEquals(map("folders", Collections.emptyList()), value(asking));
        host.close();
    }

    @Test
    void hostRefusalsBecomeHostErrorWithTheCodeNumberAndDetailsCoreSent() {
        Backend backend = Backend.create(null, null);
        TestHost host = session(3, backend);
        Set<Object> taken = new HashSet<>();
        java.util.function.Function<Map<String, Object>, HostError> answer = wire -> {
            CompletableFuture<Object> pending = backend.host().request(Protocol.HOST_WORKSPACE_LIST_CONTEXTS, null);
            Map<String, Object> request = nextRequest(host, Protocol.HOST_WORKSPACE_LIST_CONTEXTS, taken);
            host.send(map("id", request.get("id"), "error", wire));
            Throwable error = failure(pending);
            assertTrue(error instanceof HostError, String.valueOf(error));
            return (HostError) error;
        };
        for (Map<String, Object> testCase : cases("hostRequests")) {
            Map<String, Object> expected = object(testCase.get("expected"));
            if (expected.get("code") == null) {
                continue;
            }
            String name = (String) testCase.get("name");
            Map<String, Object> data = map("code", expected.get("code"));
            if (expected.get("details") != null) {
                data.put("details", expected.get("details"));
            }
            HostError error = answer.apply(map("code", expected.get("rpcCode"), "message", "Refused.", "data", data));
            assertEquals(expected.get("code"), error.code(), name);
            assertEquals(((Number) expected.get("rpcCode")).intValue(), error.rpcCode().intValue(), name);
            assertEquals(expected.get("details"), error.details(), name);
            assertEquals(Protocol.HOST_WORKSPACE_LIST_CONTEXTS, error.method(), name);
        }
        for (Map<String, Object> testCase : cases("hostFailures")) {
            Map<String, Object> expected = object(testCase.get("expected"));
            String name = (String) testCase.get("name");
            int rpcCode = ((Number) expected.get("rpcCode")).intValue();
            assertEquals(Integer.valueOf(rpcCode), HostError.CODES.get(expected.get("code")), name);
            // The number alone names the code.
            assertEquals(expected.get("code"), answer.apply(map("code", (long) rpcCode, "message", "Failed.")).code(), name);
        }
        HostError unknown = answer.apply(map("code", -32050L, "message", "Later code.",
                "data", map("details", Arrays.asList("not", "an", "object"))));
        assertEquals("plugin.host.failed", unknown.code());
        assertNull(unknown.details());
        host.close();
    }

    @Test
    void protocolOneOffersNoHostServiceAndHostRequestsStayWithinLitheHost() {
        Backend backend = Backend.create(null, null);
        TestHost host = session(1, backend);
        int before = host.messages().size();
        Throwable unavailable = failure(backend.host().credentials().get("connection.prod"));
        assertTrue(unavailable instanceof HostError);
        assertEquals("plugin.host.unavailable", ((HostError) unavailable).code());
        assertEquals(Integer.valueOf(-32601), ((HostError) unavailable).rpcCode());
        assertEquals("plugin.host.unavailable", code(backend.host().capabilities()));
        assertFalse(value(backend.host().supports(Protocol.HOST_CREDENTIALS_GET)));
        assertEquals("sdk.host.method", code(backend.host().request("lithe/function/invoke", map())));
        assertEquals(before, host.messages().size());
        host.close();

        Backend idle = Backend.create(null, null);
        assertEquals("sdk.session.notReady", code(idle.host().credentials().get("key")));
    }

    @Test
    void capabilitiesAreAskedForOnceAndSupportsReadsThem() {
        Backend backend = Backend.create(null, null);
        TestHost host = session(3, backend);
        Object answer = null;
        for (Map<String, Object> testCase : cases("hostRequests")) {
            if (object(testCase.get("expected")).get("answer") != null) {
                answer = object(testCase.get("expected")).get("answer");
                break;
            }
        }
        CompletableFuture<Boolean> first = backend.host().supports(Protocol.HOST_WORKSPACE_GET_FOLDERS);
        CompletableFuture<Boolean> second = backend.host().supports(Protocol.HOST_WORKSPACE_READ_FILE);
        Map<String, Object> request = nextRequest(host, Protocol.HOST_GET_CAPABILITIES);
        assertFalse(request.containsKey("params"));
        host.send(map("id", request.get("id"), "result", answer));
        assertEquals(Arrays.asList(true, false), Arrays.asList(value(first), value(second)));
        Map<String, Object> limits = object(value(backend.host().capabilities()).get("limits"));
        assertEquals((long) Protocol.MAX_PENDING_HOST_REQUESTS, limits.get("pendingHostRequests"));
        assertEquals(1, sent(host, Protocol.HOST_GET_CAPABILITIES).size());
        host.close();

        // Version 2 has no discovery: it reports what version 2 defines.
        Backend older = Backend.create(null, null);
        TestHost olderHost = session(2, older);
        assertTrue(value(older.host().supports(Protocol.HOST_CREDENTIALS_SET)));
        assertFalse(value(older.host().supports(Protocol.HOST_WORKSPACE_LIST_CONTEXTS)));
        olderHost.close();
    }

    @Test
    void aVersionThreeCallCarriesItsWorkspaceContextToTheHandlerOlderOnesDoNot() {
        for (Map<String, Object> testCase : cases("invocationContexts")) {
            Map<String, Object> expected = object(testCase.get("expected"));
            if (expected.get("params") == null) {
                continue;
            }
            String name = (String) testCase.get("name");
            Map<String, Object> params = object(expected.get("params"));
            AtomicReference<Object> seen = new AtomicReference<>("unset");
            Backend backend = Backend.create(null, null).function((String) params.get("functionID"), (arguments, context) -> {
                seen.set(context.workspaceContext());
                return null;
            });
            TestHost host = session(((Number) testCase.get("version")).intValue(), backend);
            Map<String, Object> response = host.request(Protocol.FUNCTION_INVOKE, params);
            assertNull(response.get("error"), name);
            assertEquals(params.get("context"), seen.get(), name);
            host.close();
        }
    }

    @Test
    void cancellingACallCancelsItsHostRequestsVersionThreeTellsCoreVersionTwoDropsTheAnswer() {
        for (int version : new int[] {3, 2}) {
            CountDownLatch started = new CountDownLatch(1);
            AtomicReference<Throwable> outcome = new AtomicReference<>();
            boolean viaGet = version == 2;
            Backend backend = Backend.create(null, null).function("secret.read", (params, context) -> {
                CompletableFuture<String> reading = context.host().credentials().get("connection.prod");
                started.countDown();
                // join() and get() wrap the failure differently; both answer as cancelled.
                try {
                    return viaGet ? reading.get() : reading.join();
                } catch (CompletionException | ExecutionException error) {
                    outcome.set(error.getCause());
                    throw error;
                }
            });
            TestHost host = session(version, backend);
            CompletableFuture<Map<String, Object>> call = host.requestAsync(Protocol.FUNCTION_INVOKE,
                    map("functionID", "secret.read", "operationID", "c"), 41L);
            Fixtures.await(started, "the host request");
            Map<String, Object> request = nextRequest(host, Protocol.HOST_CREDENTIALS_GET);
            host.notify(Protocol.CANCEL_REQUEST, map("id", 41L));
            Map<String, Object> response = Fixtures.await(call, "the cancelled call");
            assertEquals((long) Protocol.REQUEST_CANCELLED, object(response.get("error")).get("code"), "version " + version);
            // The host request fails with the call's cancellation.
            assertTrue(outcome.get() instanceof CancelledException, String.valueOf(outcome.get()));
            List<Object> cancels = sent(host, Protocol.CANCEL_REQUEST).stream().map(message -> message.get("params"))
                    .collect(Collectors.toList());
            assertEquals(version == 3 ? Arrays.asList(map("id", request.get("id"))) : Collections.emptyList(), cancels,
                    "version " + version);
            // Core's late answer is dropped; the session continues.
            host.send(map("id", request.get("id"), "error",
                    map("code", -32800L, "message", "Cancelled.", "data", map("code", "plugin.host.cancelled"))));
            assertEquals(1, host.close());
        }
    }

    @Test
    void aCancelledTokenRejectsWithItsReasonAndIsNeverSentAgain() {
        Backend backend = Backend.create(null, null);
        TestHost host = session(3, backend);
        Cancellation stop = new Cancellation();
        Host cancellable = backend.host().withCancellation(stop);
        Map<String, Object> edit = map("documentID", "d1", "expectedVersion", 4L, "edits", Collections.emptyList());
        CompletableFuture<Map<String, Object>> pending = cancellable.editor().applyEdits(CONTEXT, edit);
        Map<String, Object> request = nextRequest(host, Protocol.HOST_EDITOR_APPLY_EDITS);
        Map<String, Object> withContext = map("context", CONTEXT);
        withContext.putAll(edit);
        assertEquals(withContext, request.get("params"));
        stop.cancel("user stopped");
        Throwable error = failure(pending);
        assertTrue(error instanceof CancelledException);
        assertEquals("user stopped", error.getMessage());
        assertEquals(Arrays.asList(map("id", request.get("id"))),
                sent(host, Protocol.CANCEL_REQUEST).stream().map(message -> message.get("params")).collect(Collectors.toList()));
        host.send(map("id", request.get("id"), "result", map("documentID", "d1", "version", 5L, "applied", true)));
        assertTrue(failure(cancellable.request(Protocol.HOST_EDITOR_APPLY_EDITS, map())) instanceof CancelledException);
        assertEquals(1, sent(host, Protocol.HOST_EDITOR_APPLY_EDITS).size());
        host.close();
    }

    @Test
    void atMostSixteenHostRequestsAreUnansweredLaterOnesWaitForAFreeSlot() {
        Backend backend = Backend.create(null, null);
        TestHost host = session(3, backend);
        int max = Protocol.MAX_PENDING_HOST_REQUESTS;
        List<CompletableFuture<Map<String, Object>>> calls = new ArrayList<>();
        for (int index = 0; index < max + 2; index++) {
            calls.add(backend.host().workspace().listContexts());
        }
        // Requests with a free slot are written before listContexts returns.
        List<Map<String, Object>> first = sent(host, Protocol.HOST_WORKSPACE_LIST_CONTEXTS);
        assertEquals(max, first.size());
        Set<Object> taken = new HashSet<>();
        for (Map<String, Object> message : first) {
            taken.add(message.get("id"));
        }
        host.send(map("id", first.get(0).get("id"), "result", map("revision", 1L, "contexts", Collections.emptyList())));
        nextRequest(host, Protocol.HOST_WORKSPACE_LIST_CONTEXTS, taken);
        assertEquals(max + 1, sent(host, Protocol.HOST_WORKSPACE_LIST_CONTEXTS).size());
        // Ending the session fails the queued request too.
        host.close();
        assertEquals(map("revision", 1L, "contexts", Collections.emptyList()), value(calls.get(0)));
        for (CompletableFuture<Map<String, Object>> call : calls.subList(1, calls.size())) {
            assertEquals("sdk.session.closed", code(call));
        }
    }

    @Test
    void subscriptionEventsReachTheirListenerInOrderAlsoWhenSentBeforeTheAnswer() {
        Backend backend = Backend.create(null, null);
        TestHost host = session(3, backend);
        List<Host.Event> events = Collections.synchronizedList(new ArrayList<>());
        CountDownLatch received = new CountDownLatch(3);
        Map<String, Object> watched = map("resource", map("rootID", "r1", "path", ""), "recursive", true);
        CompletableFuture<Host.Subscription> watching = backend.host().workspace().watch(CONTEXT, watched, event -> {
            events.add(event);
            received.countDown();
        });
        Map<String, Object> request = nextRequest(host, Protocol.HOST_WORKSPACE_WATCH);
        Map<String, Object> withContext = map("context", CONTEXT);
        withContext.putAll(watched);
        assertEquals(withContext, request.get("params"));
        java.util.function.LongConsumer notify = sequence -> host.notify(Protocol.HOST_SUBSCRIPTION_NOTIFY, map(
                "subscriptionID", "s1", "sequence", sequence,
                "event", map("rootID", "r1", "changedPaths", Arrays.asList("f" + sequence), "requiresRescan", false)));
        notify.accept(1);
        host.send(map("id", request.get("id"), "result", map("subscriptionID", "s1")));
        Host.Subscription subscription = value(watching);
        notify.accept(2);
        notify.accept(4);
        Fixtures.await(received, "three events");
        List<List<Object>> seen = new ArrayList<>();
        synchronized (events) {
            for (Host.Event event : events) {
                seen.add(Arrays.asList(event.subscriptionId(), event.sequence(), event.gap(),
                        object(event.event()).get("changedPaths")));
            }
        }
        assertEquals(Arrays.asList(
                Arrays.asList("s1", 1L, false, Arrays.asList("f1")),
                Arrays.asList("s1", 2L, false, Arrays.asList("f2")),
                Arrays.asList("s1", 4L, true, Arrays.asList("f4"))), seen);
        assertEquals("s1", subscription.id());
        assertFalse(subscription.isDisposed());

        CompletableFuture<Void> disposing = subscription.dispose();
        assertTrue(subscription.isDisposed());
        Map<String, Object> dispose = nextRequest(host, Protocol.HOST_SUBSCRIPTIONS_DISPOSE);
        assertEquals(map("subscriptionID", "s1"), dispose.get("params"));
        notify.accept(5);
        host.send(map("id", dispose.get("id"), "result", null));
        value(disposing);
        // The reader handled event 5 before the dispose answer; it was dropped.
        assertEquals(3, events.size());
        host.close();
    }

    @Test
    void commandsContextChangesAndLaterHostNotificationsReachTheirHandlers() {
        List<Map<String, Object>> delivered = new ArrayList<>();
        for (Map<String, Object> testCase : cases("hostNotifications")) {
            if (Boolean.TRUE.equals(object(testCase.get("expected")).get("delivered"))
                    && !Protocol.HOST_SUBSCRIPTION_NOTIFY.equals(testCase.get("method"))) {
                delivered.add(testCase);
            }
        }
        Set<List<Object>> expected = new HashSet<>();
        for (Map<String, Object> testCase : delivered) {
            Map<String, Object> params = object(testCase.get("params"));
            expected.add(Protocol.HOST_COMMAND_DID_RUN.equals(testCase.get("method"))
                    ? Arrays.asList("command", params.get("commandID"), params.get("context"))
                    : Arrays.asList("contexts", params));
        }
        expected.add(Arrays.asList("later", map("value", 1L)));
        Set<List<Object>> seen = Collections.synchronizedSet(new HashSet<>());
        CountDownLatch all = new CountDownLatch(expected.size());
        Backend backend = Backend.create(null, null)
                .onCommand("db.refresh", (commandId, context) -> {
                    seen.add(Arrays.asList("command", commandId, context));
                    all.countDown();
                })
                .onContextsChanged(params -> {
                    seen.add(Arrays.asList("contexts", params));
                    all.countDown();
                })
                .onHostNotification("lithe/host/later/didSomething", params -> {
                    seen.add(Arrays.asList("later", params));
                    all.countDown();
                });
        assertEquals("sdk.method.reserved", assertThrows(PluginSdkError.class,
                () -> backend.onHostNotification(Protocol.HOST_COMMAND_DID_RUN, params -> {})).code());
        assertEquals("sdk.method.invalid", assertThrows(PluginSdkError.class,
                () -> backend.onHostNotification("workspace/didChange", params -> {})).code());
        assertEquals("sdk.command.invalidID", assertThrows(PluginSdkError.class,
                () -> backend.onCommand("DB_Refresh", (commandId, context) -> {})).code());
        TestHost host = session(3, backend);
        for (Map<String, Object> testCase : delivered) {
            host.notify((String) testCase.get("method"), testCase.get("params"));
        }
        // A notification this SDK does not know is ignored, as the contract asks.
        host.notify("lithe/host/unknown/ignored", map("anything", true));
        host.notify("lithe/host/later/didSomething", map("value", 1L));
        Fixtures.await(all, "the notifications");
        // Handlers run on their own threads, so only the set is fixed.
        assertEquals(expected, new HashSet<>(seen));
        assertEquals(expected.size(), seen.size());
        host.close();
    }
}
