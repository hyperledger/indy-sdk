package com.evernym.sdk.vcx.utils;

import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.ParamGuard;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.CompletableFuture;


/**
 * Created by abdussami on 17/05/18.
 */

public class UtilsApi extends VcxJava.API {
    private static final Logger logger = LoggerFactory.getLogger("UtilsApi");
    private static Callback provAsyncCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String config) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], config = [" + config + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;

            String result = config;
            future.complete(result);
        }
    };

    public static String vcxProvisionAgent(String config) {
        ParamGuard.notNullOrWhiteSpace(config, "config");
        logger.debug("vcxProvisionAgent() called with: config = [****]");
        String result = LibVcx.api.vcx_provision_agent(config);

        return result;

    }

    public static CompletableFuture<String> vcxAgentProvisionAsync(String conf) throws VcxException {
        CompletableFuture<String> future = new CompletableFuture<String>();
        logger.debug("vcxAgentProvisionAsync() called with: conf = [****]");
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_agent_provision_async(
                commandHandle, conf,
                provAsyncCB);
        checkResult(result);
        return future;
    }

    private static Callback vcxUpdateAgentInfoCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = commandHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> vcxUpdateAgentInfo(String config) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(config, "config");
        logger.debug("vcxUpdateAgentInfo() called with: config = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_agent_update_info(
                commandHandle,
                config,
                vcxUpdateAgentInfoCB
        );
        checkResult(result);
        return future;
    }

    private static Callback vcxGetMessagesCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String messages) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], messages = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            String result = messages;
            future.complete(result);
        }
    };

    public static CompletableFuture<String> vcxGetMessages(String messageStatus, String uids, String pwdids) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(messageStatus, "messageStatus");
        logger.debug("vcxGetMessages() called with: messageStatus = [" + messageStatus + "], uids = [" + uids + "], pwdids = [****]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_messages_download(
                commandHandle,
                messageStatus,
                uids,
                pwdids,
                vcxGetMessagesCB
        );
        checkResult(result);
        return future;
    }

    public static CompletableFuture<String> vcxGetAgentMessages(String messageStatus, String uids) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(messageStatus, "messageStatus");
        logger.debug("vcxGetAgentMessages() called with: messageStatus = [" + messageStatus + "], uids = [" + uids + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_download_agent_messages(
                commandHandle,
                messageStatus,
                uids,
                vcxGetMessagesCB
        );
        checkResult(result);
        return future;
    }

    private static Callback vcxUpdateMessagesCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = commandHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> vcxUpdateMessages(String messageStatus, String msgJson) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(messageStatus, "messageStatus");
        ParamGuard.notNull(msgJson, "msgJson");
        logger.debug("vcxUpdateMessages() called with: messageStatus = [" + messageStatus + "], msgJson = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_messages_update_status(
                commandHandle,
                messageStatus,
                msgJson,
                vcxUpdateMessagesCB
        );
        checkResult(result);
        return future;
    }

    private static Callback stringCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String fees) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], fees = [" + fees + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            String result = fees;
            future.complete(result);
        }
    };

    public static CompletableFuture<String> getLedgerFees() throws VcxException {
        logger.debug("getLedgerFees() called");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_ledger_get_fees(
                commandHandle,
                stringCB
        );
        checkResult(result);
        return future;
    }

    public static CompletableFuture<String> getLedgerAuthorAgreement() throws VcxException {
        logger.debug("getLedgerAuthorAgreement() called");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_get_ledger_author_agreement(
                commandHandle,
                stringCB
        );
        checkResult(result);
        return future;
    }

    public static void setActiveTxnAuthorAgreementMeta(String text, String version,
                                                         String hash, String accMechType, long timeOfAcceptance) throws VcxException {
        ParamGuard.notNull(accMechType, "accMechType");
        logger.debug("vcxProvisionAgent() called with: text = [" + text + "], version = [" + version + "]," +
                " hash = [" + hash + "], accMechType = [" + accMechType + "], timeOfAcceptance = [" + timeOfAcceptance + "]");
        int result = LibVcx.api.vcx_set_active_txn_author_agreement_meta(text, version, hash, accMechType, timeOfAcceptance);
        checkResult(result);
    }

    public static void vcxMockSetAgencyResponse(int messageIndex) {
        logger.debug("vcxMockSetAgencyResponse() called");
        LibVcx.api.vcx_set_next_agency_response(messageIndex);
    }

    public static void setPoolHandle(int handle) {
        LibVcx.api.vcx_pool_set_handle(handle);
    }

    private static Callback getReqPriceAsyncCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, long price) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], price = [" + price + "]");
            CompletableFuture<Long> future = (CompletableFuture<Long>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;

            long result = price;
            future.complete(result);
        }
    };

    public static CompletableFuture<Long> vcxGetRequestPrice(String actionJson, String requesterInfoJson) throws VcxException {
        ParamGuard.notNull(actionJson, "actionJson");
        logger.debug("vcxGetRequestPrice() called with: actionJson = [" + actionJson + "], requesterInfoJson = [" + requesterInfoJson + "]");
        CompletableFuture<Long> future = new CompletableFuture<Long>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_get_request_price(
                commandHandle, actionJson, requesterInfoJson,
                getReqPriceAsyncCB);
        checkResult(result);
        return future;
    }

    private static Callback vcxEndorseTransactionCb = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = commandHandle;
            future.complete(result);
        }
    };

    public static CompletableFuture<Integer> vcxEndorseTransaction(String transactionJson) throws VcxException {
        ParamGuard.notNull(transactionJson, "transactionJson");
        logger.debug("vcxEndorseTransaction() called with: transactionJson = [" + transactionJson + "]");
        CompletableFuture<Integer> future = new CompletableFuture<Integer>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_endorse_transaction(
                commandHandle, transactionJson,
                vcxEndorseTransactionCb);
        checkResult(result);
        return future;
    }
}
