package io.github.litheidea.plugin.sdk;

import java.io.IOException;
import java.util.List;

/**
 * What a handler gets besides its params: the call's cancellation, tools that
 * end with the call, the backend's log, and progress for plugin functions or bounded build operations.
 */
public final class CallContext {
    /** Sends one progress report; absent for model, dependency and discovery requests. */
    interface ProgressSink {
        void report(String message, Double percentage);
        default void report(String message, Double percentage, String output) { report(message, percentage); }
    }

    private final Cancellation cancellation;
    private final Backend backend;
    private final String functionId;
    private final String operationId;
    private final ProgressSink progress;

    CallContext(Cancellation cancellation, Backend backend, String functionId, String operationId, ProgressSink progress) {
        this.cancellation = cancellation;
        this.backend = backend;
        this.functionId = functionId;
        this.operationId = operationId;
        this.progress = progress;
    }

    /**
     * Cancelled when the host cancels the call, the backend shuts down, or the
     * call ends; work a handler leaves running after it returned is stopped.
     */
    public Cancellation cancellation() {
        return cancellation;
    }

    public boolean isCancelled() {
        return cancellation.isCancelled();
    }

    /** Throws {@link CancelledException} once the call was cancelled. */
    public void throwIfCancelled() {
        cancellation.throwIfCancelled();
    }

    /**
     * Runs a tool that belongs to this call: cancelling the call or ending it
     * stops the tool, and the backend shutting down ends it.
     */
    public Processes.Result run(List<String> command, Processes.Options options) throws IOException {
        Processes.Options settings = options == null ? new Processes.Options() : options.copy();
        Cancellation own = settings.cancellation();
        settings.cancellation(own == null ? cancellation : cancellation.combinedWith(own));
        return Processes.run(command, settings);
    }

    /** Runs a tool that belongs to this call with default options. */
    public Processes.Result run(List<String> command) throws IOException {
        return run(command, null);
    }

    /** Writes a diagnostic line to stderr, which the host keeps in its log. */
    public void log(String message) {
        backend.log(message);
    }

    /**
     * Reports progress of a function or bounded build call. Function messages
     * are cut to 1024 bytes, build messages to 4096; percentages are rounded to
     * 0-100. Reports after cancellation or completion are dropped.
     */
    public void progress(String message, Double percentage) { progress(message, percentage, null); }

    /** Build calls may additionally stream up to 64 KiB of output per report. */
    public void progress(String message, Double percentage, String output) {
        if (progress != null && !cancellation.isCancelled()) progress.report(message, percentage, output);
    }

    /** The function this call runs, or null for a build or LSP request. */
    public String functionId() {
        return functionId;
    }

    /** The wire operation ID of a function or bounded build call, otherwise null. */
    public String operationId() {
        return operationId;
    }
}
