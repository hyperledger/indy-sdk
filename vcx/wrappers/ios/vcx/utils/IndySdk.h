//
//  IndySdk.h
//  vcx
//
//  Created by Norman Jarvis on 2/18/19.
//  Copyright © 2019 GuestUser. All rights reserved.
//

#ifndef IndySdk_h
#define IndySdk_h

#import <Foundation/Foundation.h>
//#import "vcx.h"
//#include "vcx.h"
#import "IndyTypes.h"

@interface IndySdk : NSObject

/// Builds a TXN_AUTHR_AGRMT request. Request to add a new version of Transaction Author Agreement to the ledger.
///
/// #Params
/// text: a content of the TTA.
/// version: a version of the TTA (unique UTF-8 string).
/// requesterDID: DID of the request sender.
/// completion: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
+ (void)addTxnAuthorAgreement:(NSString *)text
                  withVersion:(NSString *)version
                fromRequester:(NSString *)requesterDID
                   completion:(void (^)(NSError *error, NSString *jsonResult))completion;


/// Builds a GET_TXN_AUTHR_AGRMT request. Request to get a specific Transaction Author Agreement from the ledger.
///
/// #Params
/// taaFilter: (Optional) specifies a condition for getting specific TAA.
///     Contains 3 mutually exclusive optional fields:
///     {
///         hash: Optional<str> - hash of requested TAA,
///         version: Optional<str> - version of requested TAA.
///         timestamp: Optional<u64> - ledger will return TAA valid at requested timestamp.
///     }
/// requesterDID: (Optional) DID of the request sender.
/// Null taaFilter or empty JSON are acceptable here. In this case, ledger will return the latest version of TAA.
///
/// completion: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
+ (void)getTxnAuthorAgreement:(NSString *)taaFilter
                fromRequester:(NSString *)requesterDID
                   completion:(void (^)(NSError *error, NSString *jsonResult))completion;


/// Builds a SET_TXN_AUTHR_AGRMT_AML request. Request to add a new list of acceptance mechanisms for transaction author agreement.
/// Acceptance Mechanism is a description of the ways how the user may accept a transaction author agreement.
///
/// #Params
/// aml: a set of new acceptance mechanisms:
/// {
///     “<acceptance mechanism label 1>”: { acceptance mechanism description 1},
///     “<acceptance mechanism label 2>”: { acceptance mechanism description 2},
///     ...
/// }
/// version: a version of new acceptance mechanisms. (Note: unique on the Ledger)
/// amlContext: (Optional) common context information about acceptance mechanisms (may be a URL to external resource).
/// requesterDID: DID of the request sender.
/// completion: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
+ (void)addAcceptanceMechanisms:(NSString *)aml
                    withVersion:(NSString *)version
                    withContext:(NSString *)amlContext
                  fromRequester:(NSString *)requesterDID
                     completion:(void (^)(NSError *error, NSString *jsonResult))completion;


/// Builds a GET_TXN_AUTHR_AGRMT_AML request. Request to get a list of  acceptance mechanisms from the ledger
/// valid for specified time or the latest one.
///
/// #Params
/// timestamp: i64 - time to get an active acceptance mechanisms. Pass -1 to get the latest one.
/// version: (Optional) version of acceptance mechanisms.
/// requesterDID: (Optional) DID of the request sender.
/// completion: Callback that takes command result as parameter.
///
/// NOTE: timestamp and version cannot be specified together.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
+ (void)getAcceptanceMechanisms:(NSNumber *)timestamp
                    withVersion:(NSString *)version
                  fromRequester:(NSString *)requesterDID
                     completion:(void (^)(NSError *error, NSString *jsonResult))completion;


/// Append transaction author agreement acceptance data to a request.
/// This function should be called before signing and sending a request
/// if there is any transaction author agreement set on the Ledger.
///
/// This function may calculate hash by itself or consume it as a parameter.
/// If all text, version and taa_digest parameters are specified, a check integrity of them will be done.
///
/// #Params
/// request_json: original request data json.
/// text and version - (optional) raw data about TAA from ledger.
///     These parameters should be passed together.
///     These parameters are required if taa_digest parameter is omitted.
/// taa_digest - (optional) hash on text and version. This parameter is required if text and version parameters are omitted.
/// mechanism - mechanism how user has accepted the TAA
/// time - UTC timestamp when user has accepted the TAA
/// completion: Callback that takes command result as parameter.
///
/// #Returns
/// Updated request result as json.
///
/// #Errors
/// Common*
+ (void)appendTxnAuthorAgreement:(NSString *)requestJson
                   withAgreement:(NSString *)text
                     withVersion:(NSString *)version
                      withDigest:(NSString *)taaDigest
                   withMechanism:(NSString *)mechanism
                   withTimestamp:(NSNumber *)time
                      completion:(void (^)(NSError *error, NSString *jsonResult))completion;


/**
 Encrypts a message by anonymous-encryption scheme.

 Sealed boxes are designed to anonymously send messages to a Recipient given its public key.
 Only the Recipient can decrypt these messages, using its private key.
 While the Recipient can verify the integrity of the message, it cannot verify the identity of the Sender.

 Note to use DID keys with this function you can call keyForDid to get key id (verkey)
 for specific DID.

 @param theirKey verkey of message recipient
 @param message a message to be encrypted
 Returns encrypted message
 */
+ (void)anonCrypt:(NSData *)message
         theirKey:(NSString *)theirKey
       completion:(void (^)(NSError *error, NSData *encryptedMsg))completion;

/**
 Decrypts a message by anonymous-encryption scheme.

 Sealed boxes are designed to anonymously send messages to a Recipient given its public key.
 Only the Recipient can decrypt these messages, using its private key.
 While the Recipient can verify the integrity of the message, it cannot verify the identity of the Sender.

 Note to use DID keys with this function you can call keyForDid to get key id (verkey)
 for specific DID.

 @param walletHandle       The wallet.
 @param myKey  Id (verkey) of my key. The key must be created by calling createKey or createAndStoreMyDid
 @param encryptedMessage encrypted message
 Returns decrypted message as an array of bytes.
 */
+ (void)anonDecrypt:(NSData *)encryptedMessage
              myKey:(NSString *)myKey
       walletHandle:(IndyHandle)walletHandle
         completion:(void (^)(NSError *error, NSData *decryptedMessage))completion;


@end

#endif /* IndySdk_h */
