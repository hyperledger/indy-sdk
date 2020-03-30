package com.evernym.sdk.vcx;


import com.evernym.sdk.vcx.connection.ConnectionErrorException;
import com.evernym.sdk.vcx.connection.InvalidConnectionHandleException;
import com.evernym.sdk.vcx.connection.InvalidInviteDetailsException;
import com.evernym.sdk.vcx.credential.BuildCredentialDefReqErrorException;
import com.evernym.sdk.vcx.credential.CreateCredentialDefException;
import com.evernym.sdk.vcx.credential.CreateCredentialRequestErrorException;
import com.evernym.sdk.vcx.credential.CredentialDefAlreadyCreatedException;
import com.evernym.sdk.vcx.credential.InvalidCredentialDefHandle;
import com.evernym.sdk.vcx.credential.InvalidCredentialDefJsonException;
import com.evernym.sdk.vcx.credential.InvalidCredentialHandleException;
import com.evernym.sdk.vcx.credential.InvalidCredentialJsonException;
import com.evernym.sdk.vcx.credential.InvalidCredentialRequestException;
import com.evernym.sdk.vcx.credential.InvalidIssuerCredentialHandleException;
import com.evernym.sdk.vcx.proof.CreateProofErrorException;
import com.evernym.sdk.vcx.proof.FailedProofComplianceException;
import com.evernym.sdk.vcx.proof.InvalidDisclosedProofHandleException;
import com.evernym.sdk.vcx.proof.InvalidProofCredentialDataException;
import com.evernym.sdk.vcx.proof.InvalidProofException;
import com.evernym.sdk.vcx.proof.InvalidProofHandleException;
import com.evernym.sdk.vcx.proof.InvalidSelfAttestedValueException;
import com.evernym.sdk.vcx.token.InsufficientTokenAmountException;
import com.evernym.sdk.vcx.utils.InvalidConfigurationException;
import com.evernym.sdk.vcx.utils.PostMsgFailureException;
import com.evernym.sdk.vcx.vcx.AlreadyInitializedException;
import com.evernym.sdk.vcx.vcx.BigNumberErrorException;
import com.evernym.sdk.vcx.vcx.CreatePoolConfigException;
import com.evernym.sdk.vcx.vcx.PoolLedgerConnectException;
import com.evernym.sdk.vcx.vcx.IndySubmitRequestErrorException;
import com.evernym.sdk.vcx.vcx.InvalidAttributeStructureException;
import com.evernym.sdk.vcx.vcx.InvalidDIDException;
import com.evernym.sdk.vcx.vcx.InvalidGenesisTxnPathException;
import com.evernym.sdk.vcx.vcx.InvalidHTTPResponseException;
import com.evernym.sdk.vcx.vcx.InvalidJsonException;
import com.evernym.sdk.vcx.vcx.InvalidKeyDelegateException;
import com.evernym.sdk.vcx.vcx.InvalidMasterSecretException;
import com.evernym.sdk.vcx.vcx.InvalidMessagesException;
import com.evernym.sdk.vcx.vcx.InvalidMsgPackException;
import com.evernym.sdk.vcx.vcx.InvalidNonceException;
import com.evernym.sdk.vcx.vcx.InvalidObjHandleException;
import com.evernym.sdk.vcx.vcx.InvalidOptionException;
import com.evernym.sdk.vcx.vcx.InvalidPredicateException;
import com.evernym.sdk.vcx.schema.InvalidSchemaCreationException;
import com.evernym.sdk.vcx.schema.InvalidSchemaException;
import com.evernym.sdk.vcx.schema.InvalidSchemaSeqNoException;
import com.evernym.sdk.vcx.schema.InvalidSchemahandleException;
import com.evernym.sdk.vcx.vcx.InvalidUrlException;
import com.evernym.sdk.vcx.vcx.InvalidVerkeyException;
import com.evernym.sdk.vcx.vcx.NoEndpointException;
import com.evernym.sdk.vcx.vcx.NoPoolOpenException;
import com.evernym.sdk.vcx.vcx.NotBase58Exception;
import com.evernym.sdk.vcx.vcx.NotReadyException;
import com.evernym.sdk.vcx.vcx.SerializationErrorException;
import com.evernym.sdk.vcx.vcx.TimeoutLibindyErrorException;
import com.evernym.sdk.vcx.vcx.UnknownErrorException;
import com.evernym.sdk.vcx.vcx.UnknownLibindyErrorException;
import com.evernym.sdk.vcx.wallet.WalletAleradyOpenException;
import com.evernym.sdk.vcx.wallet.WalletAlreadyExistsException;
import com.evernym.sdk.vcx.wallet.WalletItemAlreadyExistsException;
import com.evernym.sdk.vcx.wallet.WalletItemNotFoundException;
import com.evernym.sdk.vcx.wallet.WalletCreationException;
import com.evernym.sdk.vcx.wallet.WalletAccessFailedException;
import com.evernym.sdk.vcx.NoAgentInfoException;

import com.sun.jna.ptr.PointerByReference;
import org.json.JSONObject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Thrown when an Indy specific error has occurred.
 */
public class VcxException extends Exception {

    private static final Logger logger = LoggerFactory.getLogger("VcxException");
    private static final long serialVersionUID = 2650355290834266234L;
    private int sdkErrorCode;
    private String  sdkMessage;
    private String  sdkFullMessage;
    private String  sdkCause;
    private String sdkBacktrace;

    /**
     * Initializes a new VcxException with the specified message.
     *
     * @param message The message for the exception.
     */
    protected VcxException(String message, int sdkErrorCode) {
        super(message);
        this.sdkErrorCode = sdkErrorCode;
        setSdkErrorDetails();
    }

    private void setSdkErrorDetails(){
        PointerByReference errorDetailsJson = new PointerByReference();

        LibVcx.api.vcx_get_current_error(errorDetailsJson);

        try {
            JSONObject errorDetails = new JSONObject(errorDetailsJson.getValue().getString(0));
            this.sdkMessage = errorDetails.optString("error");
            this.sdkFullMessage = errorDetails.optString("message");
            this.sdkCause = errorDetails.optString("cause");
            this.sdkBacktrace = errorDetails.optString("backtrace");
        } catch(Exception e) {
           // TODO
           e.printStackTrace();
        }
    }

    /**
     * Gets the SDK error code for the exception.
     *
     * @return The SDK error code used to construct the exception.
     */
    public int getSdkErrorCode() {
        return sdkErrorCode;
    }

    /**
     * Gets the SDK error message for the exception.
     *
     * @return The SDK error message used to construct the exception.
     */
    public String  getSdkMessage() {return sdkMessage;}

    /**
     * Gets the SDK full error message for the exception.
     *
     * @return The SDK full error message used to construct the exception.
     */
    public String  getSdkFullMessage() {return sdkFullMessage;}

    /**
     * Gets the SDK error cause for the exception.
     *
     * @return The SDK error cause used to construct the exception.
     */
    public String  getSdkCause() {return sdkCause;}

    /**
     * Gets the SDK error backtrace for the exception.
     *
     * @return The SDK error backtrace used to construct the exception.
     */
    public String  getSdkBacktrace() {return sdkBacktrace;}

    /**
     * Initializes a new VcxException using the specified SDK error code.
     *
     * @param sdkErrorCode The SDK error code to construct the exception from.
     */
    static VcxException fromSdkError(int sdkErrorCode) {
        logger.debug("fromSdkError() called with: sdkErrorCode = [" + sdkErrorCode + "]");
        ErrorCode errorCode = ErrorCode.UNKNOWN_ERROR;
        try {
            errorCode = ErrorCode.valueOf(sdkErrorCode);
            if (errorCode == null) {
                errorCode = ErrorCode.UNKNOWN_ERROR;
            }
        } catch(Exception e) {
            //TODO: Log exception to logger
        }

        switch (errorCode) {
            case UNKNOWN_ERROR:
                return new UnknownErrorException();
            case CONNECTION_ERROR:
                return new ConnectionErrorException();
            case InvalidConnectionHandle:
                return new InvalidConnectionHandleException();
            case INVALID_CONFIGURATION:
                return new InvalidConfigurationException();
            case NOT_READY:
                return new NotReadyException();
            case NO_ENDPOINT:
                return new NoEndpointException();
            case INVALID_OPTION:
                return new InvalidOptionException();
            case INVALID_DID:
                return new InvalidDIDException();
            case INVALID_VERKEY:
                return new InvalidVerkeyException();
            case POST_MSG_FAILURE:
                return new PostMsgFailureException();
            case INVALID_NONCE:
                return new InvalidNonceException();
            case INVALID_KEY_DELEGATE:
                return new InvalidKeyDelegateException();
            case INVALID_URL:
                return new InvalidUrlException();
            case NOT_BASE58:
                return new NotBase58Exception();
            case INVALID_ISSUER_CREDENTIAL_HANDLE:
                return new InvalidIssuerCredentialHandleException();
            case INVALID_JSON:
                return new InvalidJsonException();
            case INVALID_PROOF_HANDLE:
                return new InvalidProofHandleException();
            case INVALID_CREDENTIAL_REQUEST:
                return new InvalidCredentialRequestException();
            case INVALID_MSGPACK:
                return new InvalidMsgPackException();
            case INVALID_MESSAGES:
                return new InvalidMessagesException();
            case INVALID_ATTRIBUTES_STRUCTURE:
                return new InvalidAttributeStructureException();
            case BIG_NUMBER_ERROR:
                return new BigNumberErrorException();
            case INVALID_PROOF:
                return new InvalidProofException();
            case INVALID_GENESIS_TXN_PATH:
                return new InvalidGenesisTxnPathException();
            case POOL_LEDGER_CONNECT:
                return new PoolLedgerConnectException();
            case CREATE_POOL_CONFIG:
                return new CreatePoolConfigException();
            case INVALID_PROOF_CREDENTIAL_DATA:
                return new InvalidProofCredentialDataException();
            case INDY_SUBMIT_REQUEST_ERR:
                return new IndySubmitRequestErrorException();
            case BUILD_CREDENTIAL_DEF_REQ_ERR:
                return new BuildCredentialDefReqErrorException();
            case NO_POOL_OPEN:
                return new NoPoolOpenException();
            case INVALID_SCHEMA:
                return new InvalidSchemaException();
            case FAILED_PROOF_COMPLIANCE:
                return new FailedProofComplianceException();
            case INVALID_HTTP_RESPONSE:
                return new InvalidHTTPResponseException();
            case CREATE_CREDENTIAL_DEF_ERR:
                return new CreateCredentialDefException();
            case UNKNOWN_LIBINDY_ERROR:
                return new UnknownLibindyErrorException();
            case INVALID_CREDENTIAL_DEF_JSON:
                return new InvalidCredentialDefJsonException();
            case INVALID_CREDENTIAL_DEF_HANDLE:
                return new InvalidCredentialDefHandle();
            case TIMEOUT_LIBINDY_ERROR:
                return new TimeoutLibindyErrorException();
            case CREDENTIAL_DEF_ALREADY_CREATED:
                return new CredentialDefAlreadyCreatedException();
            case INVALID_SCHEMA_SEQ_NO:
                return new InvalidSchemaSeqNoException();
            case INVALID_SCHEMA_CREATION:
                return new InvalidSchemaCreationException();
            case INVALID_SCHEMA_HANDLE:
                return new InvalidSchemahandleException();
            case INVALID_MASTER_SECRET:
                return new InvalidMasterSecretException();
            case ALREADY_INITIALIZED:
                return new AlreadyInitializedException();
            case INVALID_INVITE_DETAILS:
                return new InvalidInviteDetailsException();
            case INVALID_SELF_ATTESTED_VAL:
                return new InvalidSelfAttestedValueException();
            case INVALID_PREDICATE:
                return new InvalidPredicateException();
            case INVALID_OBJ_HANDLE:
                return new InvalidObjHandleException();
            case INVALID_DISCLOSED_PROOF_HANDLE:
                return new InvalidDisclosedProofHandleException();
            case SERIALIZATION_ERROR:
                return new SerializationErrorException();
            case WALLET_ALREADY_EXISTS:
                return new WalletAlreadyExistsException();
            case WALLET_ALREADY_OPEN:
                return new WalletAleradyOpenException();
            case WALLET_ITEM_NOT_FOUND:
                return new WalletItemNotFoundException();
            case WALLET_ITEM_CANNOT_ADD:
                return new WalletItemAlreadyExistsException();
            case INVALID_CREDENTIAL_HANDLE:
                return new InvalidCredentialHandleException();
            case INVALID_CREDENTIAL_JSON:
                return new InvalidCredentialJsonException();
            case CREATE_CREDENTIAL_REQUEST_ERROR:
                return new CreateCredentialRequestErrorException();
            case CREATE_PROOF_ERROR:
                return new CreateProofErrorException();
            case INSUFFICIENT_TOKEN_AMOUNT:
                return new InsufficientTokenAmountException();
            case ACTION_NOT_SUPPORTED:
                return new ActionNotSupportedException();
            case INVALID_WALLET_CREATION:
                return new WalletCreationException();
            case WALLET_ACCESS_FAILED:
                return new WalletAccessFailedException();
            case NO_AGENT_INFO:
                return new NoAgentInfoException();
            case UNIDENTIFIED_ERROR_CODE:
                String message = String.format("An unmapped error with the code '%s' was returned by the SDK.", sdkErrorCode);
                return new VcxException(message, sdkErrorCode);
            default:
                String errMessage = String.format("An unmapped error with the code '%s' was returned by the SDK.", sdkErrorCode);
                return new VcxException(errMessage, sdkErrorCode);
        }
    }
}


