package org.hyperledger.indy.sdk.metrics;

import com.sun.jna.Callback;
import org.hyperledger.indy.sdk.IndyException;
import org.hyperledger.indy.sdk.IndyJava;
import org.hyperledger.indy.sdk.LibIndy;
import java9.util.concurrent.CompletableFuture;

public class Metrics extends IndyJava.API {

    private Metrics() {

    }

    /*
     * STATIC CALLBACKS
     */

    /**
     * Callback used when method with string result completes
     */
    private static Callback stringCompleteCb = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommandHandle, int err, String paymentAddress) {
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(xcommandHandle);
            if (!checkResult(future, err)) return;

            future.complete(paymentAddress);
        }
    };

    /*
     * STATIC METHODS
     */

    /**
     * Collect metrics from libindy.
     *
     * @return String with a dictionary of metrics in JSON format. Where keys are names of metrics.
     * @throws IndyException Thrown if a call to the underlying SDK fails.
     */
    public static CompletableFuture<String> collectMetrics() throws IndyException {
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibIndy.api.indy_collect_metrics(
                commandHandle,
                stringCompleteCb
        );

        checkResult(future, result);

        return future;
    }
}
