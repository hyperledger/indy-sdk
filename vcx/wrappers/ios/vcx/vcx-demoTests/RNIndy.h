//
//  RNIndy.h
//  testlibvcx
//
//  Created by Norman Jarvis on 7/24/18.
//  Copyright © 2018 Norman Jarvis. All rights reserved.
//

#import <Foundation/Foundation.h>
#import "vcx.h"

@interface RNIndy : NSObject

@property (nonatomic, assign) BOOL lastDeleteWasSuccessful;
@property (nonatomic, copy) NSError* lastDeleteError;

@property (nonatomic, assign) int lastShutdownVcx;

@property (nonatomic, copy) NSString* lastOneTimeInfo;
@property (nonatomic, copy) NSError* lastOneTimeInfoError;

+(NSString*)updateInitConfig:(NSString*)config withValues:(NSString*)values;
//+(NSString*)findValue:(NSString*)values withKey:(NSString*)key;
//+(NSString*)replace:(NSString*)config findKey:(NSString*)key withValue:(NSString*)value;

-(void)simpleInit: (NSString *)config
       completion:(void (^)(BOOL success))successful;
-(void)init: (NSString *)config
    completion:(void (^)(BOOL success))successful;
-(void)getSerializedConnection: (NSInteger)connectionHandle
                    completion:(void (^)(BOOL success))successful;
-(void)deserializeConnection: (NSString *)serializedConnection
                  completion:(void (^)(BOOL success))successful;
-(void)decryptWalletFile: (NSString *) config
              completion:(void (^)(BOOL success))successful;
-(void)shutdownVcx: (BOOL *) deletePool;
-(void)credentialCreateWithMsgId: (NSString *) sourceId
            withConnectionHandle: (VcxHandle) connectionHandle
                   withMessageId: (NSString *) messageId
                      completion:(void (^)(BOOL success))successful;
-(void)serializeClaimOffer: (NSInteger)credentialHandle
                completion:(void (^)(BOOL success))successful;
-(void)deserializeClaimOffer: (NSString *)serializedCredential
                  completion:(void (^)(BOOL success))successful;
-(void)sendClaimRequest: (NSInteger) credentialHandle
   withConnectionHandle: (VcxHandle) connectionHandle
      withPaymentHandle: (vcx_payment_handle_t) paymentHandle
             completion:(void (^)(BOOL success))successful;
-(void)initWithConfig: (NSString *)config;
-(void)createOneTimeInfo: (NSString *)config
    completion:(void (^)(BOOL success))successful;
-(void)createConnectionWithInvite: (NSString *)invitationId
                    inviteDetails: (NSString *)inviteDetails
                       completion:(void (^)(BOOL success))successful;
-(void)vcxAcceptInvitation: (VcxHandle )connectionHandle
            connectionType: (NSString *)connectionType
                completion:(void (^)(BOOL success))successful;
-(void)vcxUpdatePushToken: (NSString *)config
    completion:(void (^)(BOOL success))successful;
-(NSString*)getGenesisPathWithConfig: (NSString *)config
    fileName: (NSString *)fileName;
-(void)updateClaimOfferState: (int)credentialHandle
                  completion:(void (^)(BOOL success))successful;
-(void)getClaimOfferState: (int)credentialHandle
               completion:(void (^)(BOOL success))successful;
-(void)getClaimVcx: (int)credentialHandle
        completion:(void (^)(BOOL success))successful;
-(void)exportWallet: (NSString *)exportPath
        encryptWith: (NSString *)encryptionKey
         completion:(void (^)(BOOL success))successful;
-(void)setWalletItem: (NSString *) key
          value: (NSString *) value
          completion:(void (^)(BOOL success))successful;
-(void)getWalletItem: (NSString *) key
        completion:(void (^)(BOOL success))successful;
- (void)deleteWalletItem: (NSString *) key
        completion:(void (^)(BOOL success))successful;
-(void)updateWalletItem: (NSString *) key
        value: (NSString *) value
        completion:(void (^)(BOOL success))successful;
-(void)proofCreateWithMsgId: (NSString *)sourceId
       withConnectionHandle: (vcx_connection_handle_t)connectionHandle
                  withMsgId: (NSString *)msgId
                 completion:(void (^)(BOOL success))successful;
-(void)proofRetrieveCredentials:(vcx_proof_handle_t)proofHandle
                     completion:(void (^)(BOOL success))successful;
-(void)proofGenerate:(vcx_proof_handle_t)proofHandle
    withSelectedCredentials:(NSString *)selectedCredentials
    withSelfAttestedAttrs:(NSString *)selfAttestedAttributes
          completion:(void (^)(BOOL success))successful;
-(void)proofSend:(vcx_proof_handle_t)proof_handle
    withConnectionHandle:(vcx_connection_handle_t)connection_handle
      completion:(void (^)(BOOL success))successful;
-(NSString*)createWalletKey: (NSInteger) lengthOfKey;

-(void)downloadMessages: (NSString *) messageStatus
                  uid_s: (NSString *) uid_s
                 pwdids: (NSString *) pwdids
             completion:(void (^)(BOOL success))successful;

-(void)getTokenInfo: (vcx_payment_handle_t)paymentHandle
                    completion:(void (^)(BOOL success))successful;

-(int)initNullPay;

@end
