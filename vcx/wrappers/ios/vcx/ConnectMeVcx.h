//
//  init.h
//  vcx
//
//  Created by GuestUser on 4/30/18.
//  Copyright © 2018 GuestUser. All rights reserved.
//

#ifndef init_h
#define init_h
#import "libvcx.h"

extern void VcxWrapperCommonCallback(vcx_command_handle_t xcommand_handle,
                                     vcx_error_t err);

extern void VcxWrapperCommonHandleCallback(vcx_command_handle_t xcommand_handle,
                                           vcx_error_t err,
                                           vcx_command_handle_t pool_handle);

extern void VcxWrapperCommonStringCallback(vcx_command_handle_t xcommand_handle,
                                           vcx_error_t err,
                                           const char *const arg1);

extern void VcxWrapperCommonBoolCallback(vcx_command_handle_t xcommand_handle,
                                         vcx_error_t err,
                                         unsigned int arg1);

extern void VcxWrapperCommonStringStringCallback(vcx_command_handle_t xcommand_handle,
                                                 vcx_error_t err,
                                                 const char *const arg1,
                                                 const char *const arg2);

extern void VcxWrapperCommonStringOptStringCallback(vcx_command_handle_t xcommand_handle,
                                                    vcx_error_t err,
                                                    const char *const arg1,
                                                    const char *const arg2);

extern void VcxWrapperCommonDataCallback(vcx_command_handle_t xcommand_handle,
                                         vcx_error_t err,
                                         const uint8_t *const arg1,
                                         uint32_t arg2);

extern void VcxWrapperCommonStringStringStringCallback(vcx_command_handle_t xcommand_handle,
                                                       vcx_error_t err,
                                                       const char *const arg1,
                                                       const char *const arg2,
                                                       const char *const arg3);

extern void VcxWrapperCommonStringDataCallback(vcx_command_handle_t xcommand_handle,
                                               vcx_error_t err,
                                               const char *const arg1,
                                               const uint8_t *const arg2,
                                               uint32_t arg3);

extern void VcxWrapperCommonNumberCallback(vcx_command_handle_t xcommand_handle,
                                           vcx_error_t err,
                                           vcx_command_handle_t handle);

extern void VcxWrapperCommonStringOptStringOptStringCallback(vcx_command_handle_t xcommand_handle,
                                                             vcx_error_t err,
                                                             const char *const arg1,
                                                             const char *const arg2,
                                                             const char *const arg3);

extern void VcxWrapperCommonStringStringLongCallback(vcx_command_handle_t xcommand_handle,
                                                     vcx_error_t err,
                                                     const char *arg1,
                                                     const char *arg2,
                                                     unsigned long long arg3);

extern void VcxWrapperCommonNumberStringCallback(vcx_command_handle_t xcommand_handle,
                                                 vcx_error_t err,
                                                 vcx_command_handle_t handle,
                                                 const char *const arg2);

@interface ConnectMeVcx : NSObject

- (void)initWithConfig:(NSString *)config
            completion:(void (^)(NSError *error))completion;

- (void)agentProvisionAsync:(NSString *)config
                 completion:(void (^)(NSError *error, NSString *config))completion;

- (int)updateWebhookUrl:(NSString *) notification_webhook_url;

- (NSString *)errorCMessage:(NSInteger) errorCode;

- (void)connectionCreateWithInvite:(NSString *)invitationId
                     inviteDetails:(NSString *)inviteDetails
                        completion:(void (^)(NSError *error, NSInteger connectionHandle))completion;

- (void)connectionConnect:(VcxHandle)connectionHandle
           connectionType:(NSString *)connectionType
               completion:(void (^)(NSError *error, NSString *inviteDetails))completion;

- (void)connectionGetState:(NSInteger)connectionHandle
                completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)connectionUpdateState:(NSInteger) connectionHandle
                   completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)connectionSerialize:(NSInteger)connectionHandle
                 completion:(void (^)(NSError *error, NSString *serializedConnection))completion;

- (void)connectionDeserialize:(NSString *)serializedConnection
                   completion:(void (^)(NSError *error, NSInteger connectionHandle))completion;

- (int)connectionRelease:(NSInteger) connectionHandle;

- (void)deleteConnection:(VcxHandle)connectionHandle
          withCompletion:(void (^)(NSError *error))completion;

- (void)connectionGetPwDid:(NSInteger)connectionHandle
                completion:(void (^)(NSError *error, NSString *pwDid))completion;

- (void)connectionGetTheirPwDid:(NSInteger)connectionHandle
                     completion:(void (^)(NSError *error, NSString *theirPwDid))completion;

- (void)connectionSendMessage:(VcxHandle)connectionHandle
                  withMessage:(NSString *)message
       withSendMessageOptions:(NSString *)sendMessageOptions
               withCompletion:(void (^)(NSError *error, NSString *msg_id))completion;

- (void)connectionSignData:(VcxHandle)connectionHandle
                  withData:(NSData *)dataRaw
            withCompletion:(void (^)(NSError *error, NSData *signature_raw, vcx_u32_t signature_len))completion;

- (void)connectionVerifySignature:(VcxHandle)connectionHandle
                         withData:(NSData *)dataRaw
                withSignatureData:(NSData *)signatureRaw
                   withCompletion:(void (^)(NSError *error, vcx_bool_t valid))completion;

- (void)connectionUpdateState:(VcxHandle) connectionHandle
               withCompletion:(void (^)(NSError *error, NSInteger state))completion;

- (void)connectionUpdateStateWithMessage:(VcxHandle) connectionHandle
                                 message:(NSString *)message
                          withCompletion:(void (^)(NSError *error, NSInteger state))completion;

- (void)connectionGetState:(VcxHandle) connectionHandle
            withCompletion:(void (^)(NSError *error, NSInteger state))completion;

- (void)agentUpdateInfo:(NSString *)config
             completion:(void (^)(NSError *error))completion;

- (void)getCredential:(NSInteger )credentailHandle
           completion:(void (^)(NSError *error, NSString *credential))completion;

- (void)credentialCreateWithOffer:(NSString *)sourceId
                            offer:(NSString *)credentialOffer
                       completion:(void (^)(NSError *error, NSInteger credentailHandle))completion;

- (void)credentialCreateWithMsgid:(NSString *)sourceId
                 connectionHandle:(VcxHandle)connectionHandle
                            msgId:(NSString *)msgId
                       completion:(void (^)(NSError *error, NSInteger credentialHandle, NSString *credentialOffer))completion;

- (void)credentialSendRequest:(NSInteger)credentialHandle
             connectionHandle:(VcxHandle)connectionHandle
                paymentHandle:(vcx_payment_handle_t)paymentHandle
                   completion:(void (^)(NSError *error))completion;

- (void)credentialGetState:(NSInteger )credentialHandle
                completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)credentialUpdateState:(NSInteger )credentailHandle
                completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)credentialUpdateStateWithMessage:(VcxHandle) credentialHandle
                                 message:(NSString *)message
                          withCompletion:(void (^)(NSError *error, NSInteger state))completion;

- (void)credentialGetOffers:(VcxHandle)connectionHandle
                 completion:(void (^)(NSError *error, NSString *offers))completion;

- (void)credentialSerialize:(NSInteger)credentialHandle
                 completion:(void (^)(NSError *error, NSString *state))completion;

- (void)credentialDeserialize:(NSString *)serializedCredential
                   completion:(void (^)(NSError *error, NSInteger credentialHandle))completion;

- (int)credentialRelease:(NSInteger) credentialHandle;

- (void)exportWallet:(NSString *)exportPath
         encryptWith:(NSString *)encryptionKey
          completion:(void (^)(NSError *error, NSInteger exportHandle))completion;

- (void)importWallet:(NSString *)config
           completion:(void (^)(NSError *error))completion;

- (void)addRecordWallet:(NSString *)recordType
               recordId:(NSString *)recordId
            recordValue:(NSString *)recordValue
               tagsJson:(NSString *)tagsJson
             completion:(void (^)(NSError *error))completion;

- (void)updateRecordWallet:(NSString *)recordType
              withRecordId:(NSString *)recordId
           withRecordValue:(NSString *) recordValue
            withCompletion:(void (^)(NSError *error))completion;

- (void)getRecordWallet:(NSString *)recordType
               recordId:(NSString *)recordId
            optionsJson:(NSString *)optionsJson
             completion:(void (^)(NSError *error, NSString* walletValue))completion;

- (void)deleteRecordWallet:(NSString *)recordType
            recordId:(NSString *)recordId
           completion:(void (^)(NSError *error))completion;

- (void)addRecordTagsWallet:(NSString *)recordType
                   recordId:(NSString *)recordId
                   tagsJson:(NSString *) tagsJson
                 completion:(void (^)(NSError *error))completion;

- (void)updateRecordTagsWallet:(NSString *)recordType
                      recordId:(NSString *)recordId
                      tagsJson:(NSString *) tagsJson
                    completion:(void (^)(NSError *error))completion;

- (void)deleteRecordTagsWallet:(NSString *)recordType
                      recordId:(NSString *)recordId
                  tagNamesJson:(NSString *)tagNamesJson
                    completion:(void (^)(NSError *error))completion;

- (void)openSearchWallet:(NSString *)recordType
               queryJson:(NSString *)queryJson
             optionsJson:(NSString *)optionsJson
              completion:(void (^)(NSError *error, NSInteger searchHandle))completion;

- (void)searchNextRecordsWallet:(NSInteger)searchHandle
                          count:(int)count
                     completion:(void (^)(NSError *error, NSString* records))completion;

- (void)closeSearchWallet:(NSInteger)searchHandle
               completion:(void (^)(NSError *error))completion;

- (void) proofGetRequests:(NSInteger)connectionHandle
              completion:(void (^)(NSError *error, NSString *requests))completion;

- (void) proofRetrieveCredentials:(vcx_proof_handle_t)proofHandle
                   withCompletion:(void (^)(NSError *error, NSString *matchingCredentials))completion;

- (void) proofGenerate:(vcx_proof_handle_t)proofHandle
withSelectedCredentials:(NSString *)selectedCredentials
 withSelfAttestedAttrs:(NSString *)selfAttestedAttributes
        withCompletion:(void (^)(NSError *error))completion;

- (void) proofCreateWithMsgId:(NSString *)source_id
         withConnectionHandle:(vcx_connection_handle_t)connectionHandle
                    withMsgId:(NSString *)msgId
               withCompletion:(void (^)(NSError *error, vcx_proof_handle_t proofHandle, NSString *proofRequest))completion;

- (void) proofSend:(vcx_proof_handle_t)proof_handle
withConnectionHandle:(vcx_connection_handle_t)connection_handle
    withCompletion:(void (^)(NSError *error))completion;

- (void)proofGetState:(NSInteger)proofHandle
           completion:(void (^)(NSError *error, NSInteger state))completion;

- (void)proofUpdateState:(NSInteger) proofHandle
              completion:(void (^)(NSError *error, NSInteger state))completion;

- (void) proofReject: (vcx_proof_handle_t)proof_handle
      withConnectionHandle:(vcx_connection_handle_t)connection_handle
      withCompletion: (void (^)(NSError *error))completion;

- (void) getProofMsg:(vcx_proof_handle_t) proofHandle
      withCompletion:(void (^)(NSError *error, NSString *proofMsg))completion;

- (void) getRejectMsg:(vcx_proof_handle_t) proofHandle
       withCompletion:(void (^)(NSError *error, NSString *rejectMsg))completion;

- (void)connectionRedirect:(vcx_connection_handle_t)redirect_connection_handle
      withConnectionHandle:(vcx_connection_handle_t)connection_handle
            withCompletion:(void (^)(NSError *error))completion;

- (void)getRedirectDetails:(vcx_connection_handle_t)connection_handle
      withCompletion:(void (^)(NSError *error, NSString *redirectDetails))completion;

- (void) proofCreateWithRequest:(NSString *) source_id
               withProofRequest:(NSString *) proofRequest
                 withCompletion:(void (^)(NSError *error, vcx_proof_handle_t proofHandle))completion;

- (void) proofSerialize:(vcx_proof_handle_t) proofHandle
         withCompletion:(void (^)(NSError *error, NSString *proof_request))completion;

- (void) proofDeserialize:(NSString *) serializedProof
           withCompletion:(void (^)(NSError *error, vcx_proof_handle_t proofHandle)) completion;

- (int)proofRelease:(NSInteger) proofHandle;

- (void)proofUpdateStateWithMessage:(VcxHandle) proofHandle
                            message:(NSString *)message
                     withCompletion:(void (^)(NSError *error, NSInteger state))completion;

- (int)vcxShutdown:(BOOL *)deleteWallet;

- (void)createPaymentAddress:(NSString *)seed
              withCompletion:(void (^)(NSError *error, NSString *address))completion;

- (void)getTokenInfo:(vcx_payment_handle_t)payment_handle
      withCompletion:(void (^)(NSError *error, NSString *tokenInfo))completion;

- (void)sendTokens:(vcx_payment_handle_t)payment_handle
        withTokens:(NSString *)tokens
     withRecipient:(NSString *)recipient
    withCompletion:(void (^)(NSError *error, NSString *recipient))completion;

- (void)downloadMessages:(NSString *)messageStatus
                    uid_s:(NSString *)uid_s
                  pwdids:(NSString *)pwdids
              completion:(void (^)(NSError *error, NSString* messages))completion;

- (void)updateMessages:(NSString *)messageStatus
            pwdidsJson:(NSString *)pwdidsJson
            completion:(void (^)(NSError *error))completion;

- (void)downloadAgentMessages:(NSString *)messageStatus
                    uid_s:(NSString *)uid_s
                    completion:(void (^)(NSError *error, NSString* messages))completion;

- (void) getLedgerFees:(void(^)(NSError *error, NSString *fees)) completion;

- (void) getTxnAuthorAgreement:(void(^)(NSError *error, NSString *authorAgreement)) completion;

- (vcx_error_t) activateTxnAuthorAgreement:(NSString *)text
                               withVersion:(NSString *)version
                                  withHash:(NSString *)hash
                             withMechanism:(NSString *)mechanism
                             withTimestamp:(long)timestamp;
@end

#endif /* init_h */
