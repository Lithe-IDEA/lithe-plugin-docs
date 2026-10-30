package io.github.litheidea.plugin.sdk;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.regex.Pattern;

/**
 * Wire constants and validation shared with Lithe's Rust Core
 * ({@code rust/lithe-core/src/lsp/interface/plugin_function.rs} in the Lithe
 * repository) and the Node.js
 * SDK. The SDK checks the same rules locally so a plugin author sees a mistake
 * when registering a handler instead of as a refused session at runtime.
 */
public final class Protocol {
    /** Protocol versions this SDK can answer. */
    public static final int FUNCTIONS_VERSION = 1;
    public static final int BUILD_VERSION = 10;

    public static final String INITIALIZE = "initialize";
    public static final String INITIALIZED = "initialized";
    public static final String SHUTDOWN = "shutdown";
    public static final String EXIT = "exit";
    public static final String CANCEL_REQUEST = "$/cancelRequest";
    public static final String FUNCTION_INVOKE = "lithe/function/invoke";
    public static final String FUNCTION_REGISTER = "lithe/function/register";
    public static final String FUNCTION_UNREGISTER = "lithe/function/unregister";
    public static final String FUNCTION_PROGRESS = "lithe/function/progress";
    public static final String BUILD_CONFIGURATION = "lithe/build/configuration";
    public static final String BUILD_MODEL = "lithe/build/model";
    public static final String BUILD_DEPENDENCIES = "lithe/build/dependencies";
    public static final String BUILD_EXECUTE = "lithe/build/execute";
    public static final String BUILD_TEST = "lithe/build/test";
    public static final String BUILD_DISCOVER_TESTS = "lithe/build/discoverTests";
    public static final String BUILD_PREPARE_DEBUG = "lithe/build/prepareDebug";
    public static final String BUILD_PREPARE_RUN = "lithe/build/prepareRun";
    public static final String BUILD_PROGRESS = "lithe/build/progress";

    /** JSON-RPC and LSP error codes the SDK answers with. */
    public static final int PARSE_ERROR = -32700;
    public static final int INVALID_REQUEST = -32600;
    public static final int METHOD_NOT_FOUND = -32601;
    public static final int INVALID_PARAMS = -32602;
    public static final int INTERNAL_ERROR = -32603;
    public static final int SERVER_NOT_INITIALIZED = -32002;
    public static final int REQUEST_FAILED = -32000;
    public static final int REQUEST_CANCELLED = -32800;

    public static final int MAX_FUNCTIONS = 256;
    public static final int MAX_FUNCTION_ID_BYTES = 128;
    public static final int MAX_PLUGIN_ERROR_CODE_BYTES = 64;
    public static final int MAX_MESSAGE_BYTES = 1024;
    public static final int MAX_RESULT_BYTES = 1 << 20;
    public static final int MAX_BUILD_RESULT_BYTES = 32 * 1024 * 1024;
    public static final int MAX_BUILD_MESSAGE_BYTES = 4096;
    public static final int MAX_BUILD_OUTPUT_BYTES = 65536;

    /** Minimum negotiated version; unknown methods can never pass admission. */
    static int buildMethodVersion(String method) {
        if (BUILD_CONFIGURATION.equals(method)) return 7;
        if (BUILD_MODEL.equals(method) || BUILD_DEPENDENCIES.equals(method)) return 1;
        if (BUILD_EXECUTE.equals(method)) return 2;
        if (BUILD_TEST.equals(method) || BUILD_DISCOVER_TESTS.equals(method)) return 3;
        if (BUILD_PREPARE_DEBUG.equals(method)) return 5;
        if (BUILD_PREPARE_RUN.equals(method)) return 4;
        return Integer.MAX_VALUE;
    }

    static boolean buildProgress(String method) {
        return BUILD_EXECUTE.equals(method) || BUILD_TEST.equals(method) || BUILD_PREPARE_RUN.equals(method) || BUILD_PREPARE_DEBUG.equals(method);
    }

    private static final Pattern FUNCTION_SEGMENT = Pattern.compile("[a-z][A-Za-z0-9]*");
    private static final Pattern PLUGIN_ERROR_CODE = Pattern.compile("[A-Za-z][A-Za-z0-9._-]*");

    private Protocol() {}

    /** Dot-separated lowerCamelCase segments, for example {@code snapshot.refresh}. */
    public static boolean isFunctionId(String value) {
        if (value == null || value.isEmpty() || utf8Length(value) > MAX_FUNCTION_ID_BYTES) {
            return false;
        }
        for (String segment : value.split("\\.", -1)) {
            if (!FUNCTION_SEGMENT.matcher(segment).matches()) {
                return false;
            }
        }
        return true;
    }

    /** The plugin error code alphabet Core forwards as {@code pluginErrorCode}. */
    public static boolean isPluginErrorCode(String value) {
        return value != null
                && utf8Length(value) <= MAX_PLUGIN_ERROR_CODE_BYTES
                && PLUGIN_ERROR_CODE.matcher(value).matches();
    }

    /** Cuts {@code value} to {@link #MAX_MESSAGE_BYTES} UTF-8 bytes without splitting a character. */
    public static String boundedText(String value) {
        return boundedText(value, MAX_MESSAGE_BYTES);
    }

    /** Cuts {@code value} to {@code limit} UTF-8 bytes without splitting a character. */
    public static String boundedText(String value, int limit) {
        String text = value == null ? "" : value;
        int used = 0;
        int index = 0;
        while (index < text.length()) {
            int codePoint = text.codePointAt(index);
            int size = codePoint < 0x80 ? 1 : codePoint < 0x800 ? 2 : codePoint < 0x10000 ? 3 : 4;
            if (used + size > limit) {
                return text.substring(0, index);
            }
            used += size;
            index += Character.charCount(codePoint);
        }
        return text;
    }

    /**
     * Methods reserved for the SDK. A plugin cannot install its own handler for
     * these, or send them to the host: the lifecycle belongs to the host's
     * session management, and {@code lithe/*} belongs to Lithe's typed contracts.
     */
    public static boolean isReservedMethod(String method) {
        return method.equals(INITIALIZE)
                || method.equals(INITIALIZED)
                || method.equals(SHUTDOWN)
                || method.equals(EXIT)
                || method.equals(CANCEL_REQUEST)
                || method.startsWith("lithe/");
    }

    static int utf8Length(String value) {
        return value.getBytes(StandardCharsets.UTF_8).length;
    }
    /** Strict portable user choices accepted by build protocol 10. */
    static boolean validBuildLaunchOptions(Object raw) {
        if (!(raw instanceof Map)) return false;
        Map<?, ?> value = (Map<?, ?>) raw;
        List<String> allowed = Arrays.asList("execution", "runtimeArguments", "arguments", "environment", "workingDirectory", "entrypoint");
        if (!allowed.containsAll(value.keySet()) || !Arrays.asList("application", "service", "task").contains(value.get("execution"))) return false;
        for (String name : Arrays.asList("runtimeArguments", "arguments")) {
            Object arguments = value.get(name);
            if (!(arguments instanceof List) || ((List<?>) arguments).size() > 1024) return false;
            for (Object argument : (List<?>) arguments) if (!launchText(argument, 65536)) return false;
        }
        Object environment = value.get("environment");
        if (!(environment instanceof Map) || ((Map<?, ?>) environment).size() > 64) return false;
        for (Map.Entry<?, ?> entry : ((Map<?, ?>) environment).entrySet()) {
            if (!(entry.getKey() instanceof String) || !((String) entry.getKey()).matches("[A-Za-z_][A-Za-z0-9_]{0,127}") || !launchText(entry.getValue(), 65536)) return false;
        }
        if (value.containsKey("entrypoint") && (!launchText(value.get("entrypoint"), 4096) || ((String) value.get("entrypoint")).isEmpty())) return false;
        if (value.containsKey("workingDirectory")) {
            Object rawPath = value.get("workingDirectory");
            if (!launchText(rawPath, 4096)) return false;
            String path = (String) rawPath;
            if (path.isEmpty() || path.indexOf('\\') >= 0 || path.matches("^[A-Za-z]:.*")) return false;
            if (!".".equals(path)) for (String part : path.split("/", -1)) {
                if (part.isEmpty() || ".".equals(part) || "..".equals(part)) return false;
            }
        }
        return true;
    }

    private static boolean launchText(Object value, int maximum) {
        return value instanceof String && ((String) value).getBytes(StandardCharsets.UTF_8).length <= maximum && ((String) value).indexOf(0) < 0;
    }

}
