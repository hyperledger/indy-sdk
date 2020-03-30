package com.evernym.sdk.vcx.wallet;

import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.ParamGuard;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Pointer;
import com.sun.jna.Callback;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.CompletableFuture;

public class WalletApi extends VcxJava.API {
    private static final Logger logger = LoggerFactory.getLogger("WalletApi");

    private WalletApi() {
    }

    private static Callback vcxExportWalletCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, int exportHandle) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], exportHandle = [" + exportHandle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = exportHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> exportWallet(
            String exportPath,
            String encryptionKey
    ) throws VcxException {
        ParamGuard.notNull(exportPath, "exportPath");
        ParamGuard.notNull(encryptionKey, "encryptionKey");
        logger.debug("exportWallet() called with: exportPath = [" + exportPath + "], encryptionKey = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_wallet_export(commandHandle, exportPath, encryptionKey, vcxExportWalletCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxImportWalletCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, int importHandle) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], importHandle = [" + importHandle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = importHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> importWallet(
            String config
    ) throws VcxException {
        ParamGuard.notNull(config, "config");
        logger.debug("importWallet() called with: config = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_wallet_import(commandHandle, config, vcxImportWalletCB);
        checkResult(result);

        return future;
    }

    /**
     * Callback used when bytesCb completes.
     */
    private static Callback signWithPaymentAddressCb = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommand_handle, int err, Pointer arr_raw, int arr_len) {

            CompletableFuture<byte[]> future = (CompletableFuture<byte[]>) removeFuture(xcommand_handle);
            if (! checkCallback(future, err)) return;

            byte[] result = new byte[arr_len];
            arr_raw.read(0, result, 0, arr_len);
            future.complete(result);
        }
    };

    /**
     * Signs a message with a payment address.
     *
     * @param address:  Payment address of message signer.
     * @param message   The message to be signed
     *
     * @return A future that resolves to a signature string.
     * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
     */
    public static CompletableFuture<byte[]> signWithAddress(
            String address,
            byte[] message) throws VcxException {

        ParamGuard.notNullOrWhiteSpace(address, "address");
        ParamGuard.notNull(message, "message");

        CompletableFuture<byte[]> future = new CompletableFuture<byte[]>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_wallet_sign_with_address(
                commandHandle,
                address,
                message,
                message.length,
                signWithPaymentAddressCb);

        checkResult(result);

        return future;
    }

    /**
     * Callback used when boolCb completes.
     */
    private static Callback verifyWithAddressCb = new Callback() {

        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int xcommand_handle, int err, boolean valid) {

            CompletableFuture<Boolean> future = (CompletableFuture<Boolean>) removeFuture(xcommand_handle);
            if (! checkCallback(future, err)) return;

            Boolean result = valid;
            future.complete(result);
        }
    };

    /**
     * Verify a signature with a payment address.
     *
     * @param address   Payment address of the message signer
     * @param message   Message that has been signed
     * @param signature A signature to be verified
     * @return A future that resolves to true if signature is valid, otherwise false.
     * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
     */
    public static CompletableFuture<Boolean> verifyWithAddress(
            String address,
            byte[] message,
            byte[] signature) throws VcxException {

        ParamGuard.notNullOrWhiteSpace(address, "address");
        ParamGuard.notNull(message, "message");
        ParamGuard.notNull(signature, "signature");

        CompletableFuture<Boolean> future = new CompletableFuture<Boolean>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_wallet_verify_with_address(
                commandHandle,
                address,
                message,
                message.length,
                signature,
                signature.length,
                verifyWithAddressCb);

        checkResult(result);

        return future;
    }

    private static Callback vcxAddRecordWalletCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = commandHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> addRecordWallet(
            String recordType,
            String recordId,
            String recordValue
    ) throws VcxException {
        ParamGuard.notNull(recordType, "recordType");
        ParamGuard.notNull(recordId, "recordId");
        ParamGuard.notNull(recordValue, "recordValue");
        logger.debug("addRecordWallet() called with: recordType = [" + recordType + "], recordId = [" + recordId + "], recordValue = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);
        String recordTag = "{}";

        int result = LibVcx.api.vcx_wallet_add_record(commandHandle, recordType, recordId, recordValue, recordTag, vcxAddRecordWalletCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxDeleteRecordWalletCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = commandHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> deleteRecordWallet(
            String recordType,
            String recordId
    ) throws VcxException {
        ParamGuard.notNull(recordType, "recordType");
        ParamGuard.notNull(recordId, "recordId");
        logger.debug("deleteRecordWallet() called with: recordType = [" + recordType + "], recordId = [" + recordId + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_wallet_delete_record(commandHandle, recordType, recordId, vcxDeleteRecordWalletCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxGetRecordWalletCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String recordValue) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], recordValue = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            // if nonzero errorcode, ignore walletHandle (null)
            // if error fail
            // if error = 0 then send the result
            future.complete(recordValue);
        }
    };

    public static CompletableFuture<String> getRecordWallet(
            String recordType,
            String recordId,
            String optionsJson
    ) throws VcxException {
        ParamGuard.notNull(recordType, "recordType");
        ParamGuard.notNull(recordId, "recordId");
        ParamGuard.notNull(optionsJson, "optionsJson");
        logger.debug("getRecordWallet() called with: recordType = [" + recordType + "], recordId = [" + recordId + "], optionsJson = [" + optionsJson + "]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);
        if (optionsJson.isEmpty()) optionsJson = "{}";

        int result = LibVcx.api.vcx_wallet_get_record(commandHandle, recordType, recordId, optionsJson, vcxGetRecordWalletCB);
        checkResult(result);

        return future;
    }

    private static Callback vcxUpdateRecordWalletCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = commandHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> updateRecordWallet(
            String recordType,
            String recordId,
            String recordValue
    ) throws VcxException {
        ParamGuard.notNull(recordType, "recordType");
        ParamGuard.notNull(recordId, "recordId");
        ParamGuard.notNull(recordValue, "recordValue");
        logger.debug("updateRecordWallet() called with: recordType = [" + recordType + "], recordId = [" + recordId + "], recordValue = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_wallet_update_record_value(commandHandle, recordType, recordId, recordValue, vcxUpdateRecordWalletCB);
        checkResult(result);

        return future;
    }

    public static void setWalletHandle(int handle) {
        LibVcx.api.vcx_wallet_set_handle(handle);
    }
}
