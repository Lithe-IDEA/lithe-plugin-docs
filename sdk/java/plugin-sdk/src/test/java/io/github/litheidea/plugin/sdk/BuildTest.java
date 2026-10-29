package io.github.litheidea.plugin.sdk;

import static io.github.litheidea.plugin.sdk.Fixtures.map;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;

/** Typed build operations use the same version and ownership boundaries as Core. */
@Timeout(value = 20, unit = TimeUnit.SECONDS)
class BuildTest {
    private static final Map<String, Object> LAUNCH = map("executable", map("kind", "toolchain", "value", "project-jdk"),
            "arguments", Arrays.asList("Main", "a b"), "workingDirectory", "app", "environment", map());

    private static Map<String, Object> offered(Object... versions) {
        return map("litheBuild", map("versions", Arrays.asList(versions)));
    }

    private static Object errorCode(Map<String, Object> response) {
        return Json.object(response.get("error")).get("code");
    }

    @Test
    void opaqueSelectorsCannotBeIgnoredByAnOlderBackend() {
        for (int version : new int[] {8, 9}) {
            AtomicReference<Object> seen = new AtomicReference<>();
            Backend backend = Backend.create(null, null).onTest((params, context) -> {
                seen.set(params.get("selectors")); return map("exitCode", 0, "output", "");
            });
            TestHost host = new TestHost(backend);
            try {
                host.initialize(offered(version));
                Map<String, Object> params = map("operationID", "wire-selectors", "selectorKind", "org.example.semantic",
                        "selectors", Arrays.asList("opaque # [case] () 中文"));
                Map<String, Object> result = host.request(Protocol.BUILD_TEST, params);
                if (version == 8) { assertEquals(-32602L, errorCode(result)); assertEquals(null, seen.get()); }
                else {
                    assertTrue(result.containsKey("result")); assertEquals(params.get("selectors"), seen.get());
                    params.put("suite", "Suite"); assertEquals(-32602L, errorCode(host.request(Protocol.BUILD_TEST, params)));
                    params.remove("suite"); params.put("selectors", Arrays.asList("bad\nvalue"));
                    assertEquals(-32602L, errorCode(host.request(Protocol.BUILD_TEST, params)));
                }
            } finally { host.close(); }
        }
    }

    @Test
    void eachVersionAdmitsOnlyItsTypedMethods() {
        List<String> methods = Arrays.asList("execute", "test", "discoverTests", "prepareRun", "prepareDebug", "configuration");
        int[] minimum = {2, 3, 3, 4, 5, 7};
        List<Map<String, Object>> replies = Arrays.asList(map("exitCode", 0, "output", "built\n"),
                map("exitCode", 2, "output", "test failed\n"),
                map("tests", Arrays.asList(map("suite", "Suite", "case", "exact # [case]"))), LAUNCH, map("program", "app/bin/main", "arguments", Arrays.asList("a b"), "workingDirectory", "app", "environment", map()), map("sources", Collections.emptyList()));
        for (int version = 1; version <= 9; version++) {
            Backend backend = Backend.create(null, null)
                    .onExecute((params, context) -> replies.get(0))
                    .onTest((params, context) -> replies.get(1))
                    .onDiscoverTests((params, context) -> replies.get(2))
                    .onPrepareRun((params, context) -> replies.get(3))
                    .onPrepareDebug((params, context) -> replies.get(4))
                    .onConfiguration((params, context) -> replies.get(5));
            TestHost host = new TestHost(backend);
            try {
                Map<String, Object> result = Json.object(host.initialize(offered(version, 99)).get("result"));
                Map<String, Object> experimental = Json.object(Json.object(result.get("capabilities")).get("experimental"));
                assertEquals((long) version, Json.object(experimental.get("litheBuild")).get("version"));
                assertEquals(version, backend.negotiated().buildVersion());
                for (int index = 0; index < methods.size(); index++) {
                    Map<String, Object> response = host.request("lithe/build/" + methods.get(index), map("operationID", "wire-" + index));
                    if (version < minimum[index]) {
                        assertEquals(-32601L, errorCode(response));
                    } else {
                        // Compare JSON-normalized numeric types, as seen on the wire.
                        Map<String, Object> expected = Json.object(Json.parse(Json.stringify(replies.get(index))));
                        if (index == 2 || index == 5) { expected.put("format", index == 2 ? "litheTestDiscovery" : "litheBuildConfiguration"); expected.put("version", 1L); }
                        assertEquals(expected, response.get("result"));
                    }
                }
            } finally { host.close(); }
        }
    }

    @Test
    void inlineReportsNeedVersionEightAndRemainBoundToTheCompletingTest() {
        for (int version : new int[] {7, 8}) {
            java.util.concurrent.atomic.AtomicInteger calls = new java.util.concurrent.atomic.AtomicInteger();
            Map<String, Object> report = map("format", "litheTestResults", "version", 1L, "tests", Arrays.asList(
                    map("project", "a", "suite", "Same", "case", "checks", "status", "failed"),
                    map("project", "b", "suite", "Same", "case", "checks", "status", "passed")));
            Backend backend = Backend.create(null, null).onTest((params, context) -> {
                calls.incrementAndGet(); return map("exitCode", 1, "output", "failed", "testResults", report);
            });
            TestHost host = new TestHost(backend);
            try {
                host.initialize(offered(version));
                Map<String, Object> response = host.request("lithe/build/test", map("operationID", "wire-reports", "testResults", true));
                if (version == 7) { assertEquals(-32602L, errorCode(response)); assertEquals(0, calls.get()); }
                else { assertEquals(report, Json.object(response.get("result")).get("testResults")); assertEquals(1, calls.get()); }
            } finally { host.close(); }
        }
    }

    @Test
    void readOnlyModelInputsRequireVersionSixAndPreserveHostIdentity() {
        Map<String, Object> inputs = map("version", 1L, "revision", 17L, "inputs", Arrays.asList(
                map("pluginID", "dev.source", "buildSystemID", "source", "state", "unavailable", "instances", Collections.emptyList())));
        for (int version : new int[] {5, 6, 7, 8}) {
            AtomicReference<Object> seen = new AtomicReference<>();
            Backend backend = Backend.create(null, null).onModel((params, context) -> {
                seen.set(params.get("modelInputs"));
                return map("projects", Collections.emptyList(), "targets", Collections.emptyList(), "tasks", Collections.emptyList());
            });
            TestHost host = new TestHost(backend);
            try {
                host.initialize(offered(version));
                Map<String, Object> response = host.request("lithe/build/model", map("modelInputs", inputs));
                if (version == 5) { assertEquals(-32602L, errorCode(response)); assertEquals(null, seen.get()); }
                else { assertEquals(inputs, seen.get()); assertTrue(response.containsKey("result")); }
            } finally { host.close(); }
        }
    }

    @Test
    void negotiationSelectsOnlyACommonIntegerVersion() {
        for (Object[] versions : new Object[][] {{1, 4, 3, 2}, {99, "4", true}, {}}) {
            Backend backend = Backend.create(null, null).onExecute((params, context) -> map("exitCode", 0, "output", ""));
            TestHost host = new TestHost(backend);
            try {
                host.initialize(offered(versions));
                assertEquals(versions.length == 4 ? 4 : 0, backend.negotiated().buildVersion());
            } finally { host.close(); }
        }
    }

    @Test
    void progressUsesTheWireIdAndStopsWhenTheCallEnds() throws Exception {
        CountDownLatch entered = new CountDownLatch(1);
        CountDownLatch release = new CountDownLatch(1);
        AtomicReference<CallContext> late = new AtomicReference<>();
        Backend backend = Backend.create(null, null).onPrepareRun((params, context) -> {
            assertEquals(params.get("operationID"), context.operationId());
            late.set(context);
            context.progress(repeat("😀", 2000), 140.0, repeat("界", 30000));
            entered.countDown();
            assertTrue(release.await(5, TimeUnit.SECONDS), "preparation was not released");
            return LAUNCH;
        });
        TestHost host = new TestHost(backend);
        try {
            host.initialize(offered(4));
            CompletableFuture<Map<String, Object>> pending = host.requestAsync("lithe/build/prepareRun", map("operationID", "wire-42"), 42L);
            assertTrue(entered.await(5, TimeUnit.SECONDS));
            Map<String, Object> report = Json.object(host.waitFor(message -> Protocol.BUILD_PROGRESS.equals(message.get("method")), "progress").get("params"));
            assertEquals("wire-42", report.get("operationID"));
            assertEquals(4096, Protocol.utf8Length((String) report.get("message")));
            assertEquals(65535, Protocol.utf8Length((String) report.get("output")));
            assertEquals(100L, report.get("percentage"));
            release.countDown();
            assertEquals(LAUNCH, Fixtures.await(pending, "preparation to finish").get("result"));
            late.get().progress("late", null, "late output");
            assertEquals(1L, host.messages().stream().filter(message -> Protocol.BUILD_PROGRESS.equals(message.get("method"))).count());
        } finally { release.countDown(); host.close(); }
    }

    @Test
    void nativeDebugProgressUsesWireIdentityAndStopsAfterCompletion() throws Exception {
        CountDownLatch entered = new CountDownLatch(1);
        CountDownLatch release = new CountDownLatch(1);
        AtomicReference<CallContext> late = new AtomicReference<>();
        Backend backend = Backend.create(null, null).onPrepareDebug((params, context) -> {
            assertEquals(params.get("operationID"), context.operationId());
            late.set(context);
            context.progress(repeat("😀", 2000), 140.0, repeat("界", 30000));
            entered.countDown();
            assertTrue(release.await(5, TimeUnit.SECONDS), "preparation was not released");
            return map("program", "app/bin/main", "arguments", Collections.emptyList(), "workingDirectory", "app", "environment", map());
        });
        TestHost host = new TestHost(backend);
        try {
            host.initialize(offered(5));
            CompletableFuture<Map<String, Object>> pending = host.requestAsync("lithe/build/prepareDebug", map("operationID", "wire-42"), 42L);
            assertTrue(entered.await(5, TimeUnit.SECONDS));
            Map<String, Object> report = Json.object(host.waitFor(message -> Protocol.BUILD_PROGRESS.equals(message.get("method")), "progress").get("params"));
            assertEquals("wire-42", report.get("operationID"));
            assertEquals(4096, Protocol.utf8Length((String) report.get("message")));
            assertEquals(65535, Protocol.utf8Length((String) report.get("output")));
            assertEquals(100L, report.get("percentage"));
            release.countDown();
            assertTrue(Json.object(Fixtures.await(pending, "preparation to finish").get("result")).containsKey("program"));
            late.get().progress("late", null, "late output");
            assertEquals(1L, host.messages().stream().filter(message -> Protocol.BUILD_PROGRESS.equals(message.get("method"))).count());
        } finally { release.countDown(); host.close(); }
    }

    @Test
    void cancellingOneBuildDoesNotCancelAnotherAndDropsCancelledProgress() throws Exception {
        CountDownLatch entered = new CountDownLatch(2);
        CountDownLatch release = new CountDownLatch(1);
        Backend backend = Backend.create(null, null).onExecute((params, context) -> {
            entered.countDown();
            assertTrue(release.await(5, TimeUnit.SECONDS));
            context.progress("cancelled progress", null);
            return map("exitCode", 0, "output", "");
        }).onTest((params, context) -> {
            entered.countDown();
            assertTrue(release.await(5, TimeUnit.SECONDS));
            return map("exitCode", 3, "output", "independent failure");
        });
        TestHost host = new TestHost(backend);
        try {
            host.initialize(offered(4));
            CompletableFuture<Map<String, Object>> pending = host.requestAsync("lithe/build/execute", map("operationID", "wire-cancel"), 42L);
            CompletableFuture<Map<String, Object>> sibling = host.requestAsync("lithe/build/test", map("operationID", "wire-other"), 43L);
            assertTrue(entered.await(5, TimeUnit.SECONDS));
            host.notify("$/cancelRequest", map("id", 42L));
            // The barrier response proves the read loop saw cancellation before
            // releasing the first handler; no timing sleep chooses the race.
            Map<String, Object> barrier = host.request("custom.barrier", map());
            assertEquals(-32601L, errorCode(barrier));
            release.countDown();
            assertEquals(-32800L, errorCode(Fixtures.await(pending, "cancelled execution")));
            assertEquals(map("exitCode", 3L, "output", "independent failure"), Fixtures.await(sibling, "sibling execution").get("result"));
            assertFalse(host.messages().stream().anyMatch(message -> Protocol.BUILD_PROGRESS.equals(message.get("method"))));
        } finally { release.countDown(); host.close(); }
    }

    @Test
    void invalidRepliesAndParametersDoNotEndTheSession() {
        AtomicReference<Object> reply = new AtomicReference<>();
        Backend backend = Backend.create(null, null).onExecute((params, context) -> reply.get())
                .onDiscoverTests((params, context) -> map("format", "wrong", "tests", Collections.emptyList()))
                .onPrepareRun((params, context) -> map("executable", false));
        TestHost host = new TestHost(backend);
        try {
            host.initialize(offered(4));
            assertEquals(-32602L, errorCode(host.request("lithe/build/execute", map())));
            assertEquals(-32602L, errorCode(host.request("lithe/build/execute", Collections.emptyList())));
            for (Object value : Arrays.asList(map(), map("exitCode", true, "output", ""),
                    map("exitCode", 2147483648L, "output", ""), map("exitCode", 0, "output", false))) {
                reply.set(value);
                assertEquals(-32603L, errorCode(host.request("lithe/build/execute", map("operationID", "wire-invalid"))));
            }
            for (String method : Arrays.asList("discoverTests", "prepareRun")) {
                assertEquals(-32603L, errorCode(host.request("lithe/build/" + method, map("operationID", "wire-invalid"))));
            }
            reply.set(map("exitCode", 7, "output", "tool failure is a result"));
            assertEquals(map("exitCode", 7L, "output", "tool failure is a result"),
                    host.request("lithe/build/execute", map("operationID", "wire-valid")).get("result"));
        } finally { host.close(); }
    }

    private static String repeat(String text, int count) {
        StringBuilder result = new StringBuilder();
        for (int index = 0; index < count; index++) result.append(text);
        return result.toString();
    }
}
