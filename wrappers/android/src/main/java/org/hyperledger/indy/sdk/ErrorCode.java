package org.hyperledger.indy.sdk;

import java.util.HashMap;
import java.util.Map;

/**
 * Enumeration of error codes returned by the indy SDK.
 */
public enum ErrorCode {

	/**
	 * Success
	 */
	Success(0),

	// Common errors

	/**
	 * Caller passed invalid value as param 1 (null, invalid json and etc..)
	 */
	CommonInvalidParam1(100),

	/**
	 * Caller passed invalid value as param 2 (null, invalid json and etc..)
	 */
	CommonInvalidParam2(101),

	/**
	 * Caller passed invalid value as param 3 (null, invalid json and etc..)
	 */
	CommonInvalidParam3(102),

	/**
	 * Caller passed invalid value as param 4 (null, invalid json and etc..)
	 */
	CommonInvalidParam4(103),

	/**
	 * Caller passed invalid value as param 5 (null, invalid json and etc..)
	 */
	CommonInvalidParam5(104),
 
	/**
	 * Caller passed invalid value as param 6 (null, invalid json and etc..)
	 */
	CommonInvalidParam6(105),

	/**
	 * Caller passed invalid value as param 7 (null, invalid json and etc..)
	 */
	CommonInvalidParam7(106),

	/**
	 * Caller passed invalid value as param 8 (null, invalid json and etc..)
	 */
	CommonInvalidParam8(107),

	/**
	 * Caller passed invalid value as param 9 (null, invalid json and etc..)
	 */
	CommonInvalidParam9(108),
 
	/**
	 * Caller passed invalid value as param 10 (null, invalid json and etc..)
	 */
	CommonInvalidParam10(109),
 
	/**
	 * Caller passed invalid value as param 11 (null, invalid json and etc..)
	 */
	CommonInvalidParam11(110),

	/**
	 * Caller passed invalid value as param 12 (null, invalid json and etc..)
	 */
	CommonInvalidParam12(111),

	/**
	 * Invalid library state was detected in runtime. It signals library bug
	 */
	CommonInvalidState(112),
 
	/**
	 * Object (json, config, key, credential and etc...) passed by library caller has invalid structure
	 */
	CommonInvalidStructure(113),

	/**
	 * IO Error
	 */
	CommonIOError(114),

	/**
	 * Caller passed invalid value as param 13 (null, invalid json and etc..)
	 */
	CommonInvalidParam13(115),

	/**
	 * Caller passed invalid value as param 14 (null, invalid json and etc..)
	 */
	CommonInvalidParam14(116),

	// Wallet errors
	 
	/**
	 * Caller passed invalid wallet handle
	 */
	WalletInvalidHandle(200),

	/**
	 * Unknown type of wallet was passed on create_wallet
	 */
	WalletUnknownTypeError(201),

	/**
	 * Attempt to register already existing wallet type
	 */
	WalletTypeAlreadyRegisteredError(202),

	/**
	 * Attempt to create wallet with name used for another exists wallet
	 */
	WalletAlreadyExistsError(203),
 
	/**
	 * Requested entity id isn't present in wallet
	 */
	WalletNotFoundError(204),
 
	/**
	 * Trying to use wallet with pool that has different name
	 */
	WalletIncompatiblePoolError(205),

	/**
	 * Trying to open wallet that was opened already
	 */
	WalletAlreadyOpenedError(206),

	/**
	 * Attempt to open encrypted wallet with invalid credentials
	 */
	WalletAccessFailed(207),

	/**
	 * Input provided to wallet operations is considered not valid
	 */
	WalletInputError(208),

	/**
	 * Decoding of wallet data during input/output failed
	 */
	WalletDecodingError(209),

	/**
	 * Storage error occurred during wallet operation
	 */
	WalletStorageError(210),

	/**
	 * Error during encryption-related operations
	 */
	WalletEncryptionError(211),

	/**
	 * Requested wallet item not found
	 */
	WalletItemNotFound(212),

	/**
	 * Returned if wallet's add_record operation is used with record name that already exists
	 */
	WalletItemAlreadyExists(213),

	/**
	 * Returned if provided wallet query is invalid
	 */
	WalletQueryError(214),

	// Ledger errors
	
	/**
	 * Trying to open pool ledger that wasn't created before
	 */
	PoolLedgerNotCreatedError(300),

	/**
	 * Caller passed invalid pool ledger handle
	 */
	PoolLedgerInvalidPoolHandle(301),

	/**
	 * Pool ledger terminated
	 */
	PoolLedgerTerminated(302),

	/**
	 *  No consensus during ledger operation
	 */
	LedgerNoConsensusError(303),

	/**
	 * Attempt to parse invalid transaction response
	 */
	LedgerInvalidTransaction(304),

	/**
	 * Attempt to send transaction without the necessary privileges
	 */
	LedgerSecurityError(305),

	/**
	 * Attempt to create pool ledger config with name used for another existing pool
	 */
	PoolLedgerConfigAlreadyExistsError(306),

	/**
	 * Timeout for action
	 */
	PoolLedgerTimeout(307),

	/**
	 * Attempt to open Pool for witch Genesis Transactions are not compatible with set Protocol version.
	 * Call pool.indy_set_protocol_version to set correct Protocol version.
	 */
	PoolIncompatibleProtocolVersion(308),

	/**
	 * Item not found on ledger.
	 */
	LedgerNotFound(309),

	// Crypto errors

	/**
	 * Revocation registry is full and creation of new registry is necessary
	 */
	AnoncredsRevocationRegistryFullError(400),

	/**
	 * ???
	 */
	AnoncredsInvalidUserRevocId(401),

	/**
	 * Attempt to generate master secret with duplicated name
	 */
	AnoncredsMasterSecretDuplicateNameError(404),

	/**
	 * ???
	 */
	AnoncredsProofRejected(405),
	
	/**
	 * Attempt to use a revoked credential.
	 */
	AnoncredsCredentialRevoked(406),

	/**
	 * Attempt to create credential definition with duplicated did schema pair.
	 */
	AnoncredsCredDefAlreadyExistsError(407),

	// Crypto errors
	
	/**
	 * Unknown format of DID entity keys
	 */
	UnknownCryptoTypeError(500),

	/**
	 * Attempt to create duplicate did.
	 */
	DidAlreadyExistsError(600),

	/**
	 * Unknown payment method has been called
	 */
	UnknownPaymentMethod(700),

	/**
	 * No method were scraped from inputs/outputs or more than one were scraped
	 */
	IncompatiblePaymentError(701),

	/**
	 * Insufficient funds on inputs
	 */
	InsufficientFundsError(702),

	/**
	 * No such source on a ledger
	 */
	PaymentSourceDoesNotExistError(703),

	/**
	 * Operation is not supported for payment method
	 */
	PaymentOperationNotSupportedError(704),

	/**
	 * Extra funds on inputs
	 */
	ExtraFundsError(705),

	/**
	 * The transaction is not allowed to a requester
	 */
	TransactionNotAllowedError(706),
	;

	private int value;
	private static Map<Integer, ErrorCode> map = new HashMap<Integer, ErrorCode>();
	
	private ErrorCode(int value) {

		this.value = value;
	}

	static {

		for (ErrorCode errorCode : ErrorCode.values()) {

			map.put(Integer.valueOf(errorCode.value), errorCode);
		}
	}

	/**
	 * Gets the ErrorCode that corresponds to the specified int value.
	 * 
	 * @param value	The integer to get the error code for.
	 * @return	The ErrorCode that corresponds to the specified integer.
	 */
	public static ErrorCode valueOf(int value) {

		return map.get(Integer.valueOf(value));
	}

	/**
	 * Gets the integer value for a specific ErrorCode.
	 * 
	 * @return The integer value of the ErrorCode.
	 */
	public int value() {

		return this.value;
	}
}
