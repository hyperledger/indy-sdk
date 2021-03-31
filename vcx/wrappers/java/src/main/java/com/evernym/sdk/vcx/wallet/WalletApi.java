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
            String recordValue,
            String tagsJson
    ) throws VcxException {
        ParamGuard.notNull(recordType, "recordType");
        ParamGuard.notNull(recordId, "recordId");
        ParamGuard.notNull(recordValue, "recordValue");
        logger.debug("addRecordWallet() called with: recordType = [" + recordType + "], recordId = [" + recordId + "], recordValue = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);
        if (tagsJson.isEmpty()) tagsJson = "{}";

        int result = LibVcx.api.vcx_wallet_add_record(commandHandle, recordType, recordId, recordValue, tagsJson, vcxAddRecordWalletCB);
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

    /**
     * Callback used when function returning void completes.
     */
    private static Callback vcxAddRecordTagsWalletCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = commandHandle;
            future.complete(result);
        }
    };


    /**
     * Add tags to a record already stored in the storage wallet.
     *
     * @param recordType Allows to separate different record types collections
     * @param recordId The id of record
     * @param tagsJson The record tags used for search and storing meta information as json:
     *                  {
     *                      "tagName1": "str", // string tag (will be stored encrypted)
     *                      "~tagName2": "str", // string tag (will be stored un-encrypted)
     *                  }
     * @return A future that resolves no value.
     * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
     */
    public static CompletableFuture<Integer> addRecordTagsWallet(
            String recordType,
            String recordId,
            String tagsJson
    ) throws VcxException {
        ParamGuard.notNull(recordType, "recordType");
        ParamGuard.notNull(recordId, "recordId");
        ParamGuard.notNull(tagsJson, "tagsJson");
        logger.debug("addRecordTagsWallet() called with: recordType = [" + recordType + "], recordId = [" + recordId + "], tagsJson = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_wallet_add_record_tags(commandHandle, recordType, recordId, tagsJson, vcxAddRecordTagsWalletCB);
        checkResult(result);

        return future;
    }

    /**
     * Callback used when function returning void completes.
     */
    private static Callback vcxUpdateRecordTagsWalletCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = commandHandle;
            future.complete(result);
        }
    };

    /**
     * Update tags on a record, removing any previous value.
     *
     * @param recordType Allows to separate different record types collections
     * @param recordId The id of record
     * @param tagsJson The record tags used for search and storing meta information as json:
     *                  {
     *                      "tagName1": "str", // string tag (will be stored encrypted)
     *                      "~tagName2": "str", // string tag (will be stored un-encrypted)
     *                  }
     * @return A future that resolves no value.
     * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
     */
    public static CompletableFuture<Integer> updateRecordTagsWallet(
            String recordType,
            String recordId,
            String tagsJson
    ) throws VcxException {
        ParamGuard.notNull(recordType, "recordType");
        ParamGuard.notNull(recordId, "recordId");
        ParamGuard.notNull(tagsJson, "tagsJson");
        logger.debug("updateRecordTagsWallet() called with: recordType = [" + recordType + "], recordId = [" + recordId + "], tagsJson = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_wallet_update_record_tags(commandHandle, recordType, recordId, tagsJson, vcxUpdateRecordTagsWalletCB);
        checkResult(result);

        return future;
    }

    /**
     * Callback used when function returning void completes.
     */
    private static Callback vcxDeleteRecordTagsWalletCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = commandHandle;
            future.complete(result);
        }
    };

    /**
     * Delete tags associated with a record
     *
     * @param recordType Allows to separate different record types collections
     * @param recordId The id of record
     * @param tagNamesJson The list of tag names to remove from the record as json array:
     *                     ["tagName1", "tagName2", ...]
     * @return A future that resolves no value.
     * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
     */
    public static CompletableFuture<Integer> deleteRecordTagsWallet(
            String recordType,
            String recordId,
            String tagNamesJson
    ) throws VcxException {
        ParamGuard.notNull(recordType, "recordType");
        ParamGuard.notNull(recordId, "recordId");
        ParamGuard.notNull(tagNamesJson, "tagNamesJson");
        logger.debug("deleteRecordTagsWallet() called with: recordType = [" + recordType + "], recordId = [" + recordId + "], tagsNamesJson = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_wallet_delete_record_tags(commandHandle, recordType, recordId, tagNamesJson, vcxDeleteRecordTagsWalletCB);
        checkResult(result);

        return future;
    }

    /**
     * Callback used when function returning integer completes.
     */
    private static Callback vcxOpenSearchWalletCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, int searchHandle) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], searchHandle = [" + searchHandle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (! checkCallback(future, err)) return;
            Integer result = searchHandle;
            future.complete(result);
        }
    };

    /**
     * Open a search handle within the storage wallet.
     *
     * @param recordType Allows to separate different record types collections
     * @param queryJson MongoDB style query to wallet record tags:
     *                    {
     *                      "tagName": "tagValue",
     *                      $or: {
     *                          "tagName2": { $regex: 'pattern' },
     *                          "tagName3": { $gte: '123' },
     *                      }
     *                    }
     * @param optionsJson {
     *                      retrieveRecords: (optional, true by default) If false only "counts" will be calculated,
     *                      retrieveTotalCount: (optional, false by default) Calculate total count,
     *                      retrieveType: (optional, false by default) Retrieve record type,
     *                      retrieveValue: (optional, true by default) Retrieve record value,
     *                      retrieveTags: (optional, false by default) Retrieve record tags,
     *                    }
     * @return A future that resolves to WalletSearch instance.
     * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
     */
    public static CompletableFuture<Integer> openSearchWallet(
            String recordType,
            String queryJson,
            String optionsJson) throws VcxException {
        ParamGuard.notNull(recordType, "recordType");
        ParamGuard.notNull(queryJson, "queryJson");
        ParamGuard.notNull(optionsJson, "optionsJson");
        logger.debug("openSearchWallet() called with: recordType = [" + recordType + "], queryJson = [" + queryJson + "], optionsJson = [" + optionsJson + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);
        if (queryJson.isEmpty()) queryJson = "{}";
        if (optionsJson.isEmpty()) optionsJson = "{}";

        int result = LibVcx.api.vcx_wallet_open_search(
                commandHandle,
                recordType,
                queryJson,
                optionsJson,
                vcxOpenSearchWalletCB
        );
        checkResult(result);
        return future;
    }

    /**
     * Callback used when function returning string completes.
     */
    private static Callback vcxSearchNextRecordsWalletCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String recordValue) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], recordValue = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(recordValue);
        }
    };

    /**
     * Search for next n record from an open search handle
     *
     * @param searchHandle The wallet search handle.
     * @param count Count of records to fetch
     * @return A future resolving to the wallet records json:
     * {
     *      totalCount: int, // present only if retrieveTotalCount set to true
     *      records: [{ // present only if retrieveRecords set to true
     *          id: "Some id",
     *          type: "Some type", // present only if retrieveType set to true
     *          value: "Some value", // present only if retrieveValue set to true
     *          tags: "Some tags json", // present only if retrieveTags set to true
     *      }],
     * }
     * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
     */
    public static CompletableFuture<String> searchNextRecordsWallet(
            int searchHandle,
            int count
    ) throws VcxException {
        ParamGuard.notNull(searchHandle, "searchHandle");
        logger.debug("searchNextRecordsWallet() called with: searchHandle = [" + searchHandle + "], count = [" + count + "]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_wallet_search_next_records(commandHandle, searchHandle, count, vcxSearchNextRecordsWalletCB);
        checkResult(result);

        return future;
    }

    /**
     * Callback used when function returning void completes.
     */
    private static Callback vcxCloseSearchWalletCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = commandHandle;
            future.complete(result);
        }
    };

    /**
     * Close a search
     *
     * @param searchHandle The wallet search handle.
     * @return A future resolving to no value.
     * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
     */
    public static CompletableFuture<Integer> closeSearchWallet(
            int searchHandle
    ) throws VcxException {
        ParamGuard.notNull(searchHandle, "searchHandle");
        logger.debug("closeSearchWallet() called with: searchHandle = [" + searchHandle + "]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_wallet_close_search(commandHandle, searchHandle, vcxCloseSearchWalletCB);
        checkResult(result);

        return future;
    }

    public static void setWalletHandle(int handle) {
        LibVcx.api.vcx_wallet_set_handle(handle);
    }
}
