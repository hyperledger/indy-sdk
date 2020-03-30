//
//  RNIndy.m
//  testlibvcx
//
//  Created by Norman Jarvis on 7/24/18.
//  Copyright © 2018 Norman Jarvis. All rights reserved.
//

#import "RNIndy.h"

@implementation RNIndy

+(NSString*)updateInitConfig:(NSString*)config
    withValues:(NSString*)values
{
    NSError* error;
    NSMutableDictionary *parsedValues;
    if (values == nil) {
        parsedValues = [NSMutableDictionary dictionary];
    } else {
        parsedValues = [NSJSONSerialization JSONObjectWithData:[values dataUsingEncoding:NSUTF8StringEncoding] options:NSJSONReadingMutableContainers error:&error];
    }
    NSMutableDictionary *currentConfig = [NSJSONSerialization JSONObjectWithData:[config dataUsingEncoding:NSUTF8StringEncoding] options:NSJSONReadingMutableContainers error:&error];
    for (NSString *obj in parsedValues)
    {
        if (![obj isEqualToString:@"institution_name"] &&
            ![obj isEqualToString:@"institution_logo_url"] &&
            ![obj isEqualToString:@"genesis_path"])
        {
            currentConfig[obj] = parsedValues[obj];
        }
    }

    NSData *jsonData = [NSJSONSerialization dataWithJSONObject:currentConfig
       options:(NSJSONWritingOptions) (0)
         error:&error];
    return [[NSString alloc] initWithData:jsonData encoding:NSUTF8StringEncoding];
}

-(void)simpleInit: (NSString *)config
       completion:(void (^)(BOOL success))successful
{

    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
        [[[ConnectMeVcx alloc] init] initWithConfig:config completion:^(NSError *error) {
            if (error != nil && error.code != 0 && error.code != 1044)
            {
                NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while initializing vcx: %@ :: %ld", error.domain, (long)error.code];
                NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
                //return @false;
                dispatch_async(dispatch_get_main_queue(), ^{
                    successful(NO);
                });
            }else{
                NSLog(@"init was successful!");
                //return @true;
                dispatch_async(dispatch_get_main_queue(), ^{
                    successful(YES);
                });
            }
        }];
    });
}

-(void)init: (NSString *)config
    completion:(void (^)(BOOL success))successful
{

    config = [RNIndy updateInitConfig:config withValues:[self lastOneTimeInfo]];

    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
        [[[ConnectMeVcx alloc] init] initWithConfig:config completion:^(NSError *error) {
            if (error != nil && error.code != 0 && error.code != 1044)
            {
                NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while initializing vcx: %@ :: %ld", error.domain, (long)error.code];
                NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
                //return @false;
                dispatch_async(dispatch_get_main_queue(), ^{
                    successful(NO);
                });
            }else{
                NSLog(@"init was successful!");
                //return @true;
                dispatch_async(dispatch_get_main_queue(), ^{
                    successful(YES);
                });
            }
        }];
    });
}

-(void)getSerializedConnection: (NSInteger)connectionHandle
                    completion:(void (^)(BOOL success))successful
{
    // TODO call vcx_connection_serialize and pass connectionHandle
    // it would return a string
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] connectionSerialize:connectionHandle
      completion:^(NSError *error, NSString *state) {
          if (error != nil && error.code != 0)
          {
              NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while serializing connection handle: %@ :: %ld", error.domain, (long)error.code];
              NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
              //throw error;
              //return @"";
              dispatch_async(dispatch_get_main_queue(), ^{
                  successful(NO);
              });
          }else{
              NSLog(@"getSerializedConnection was successful!");
              //resolve(state);
              //return state;
              dispatch_async(dispatch_get_main_queue(), ^{
                  successful(YES);
              });
          }
      }];
    });
}

-(void)deserializeConnection: (NSString *)serializedConnection
                  completion:(void (^)(BOOL success))successful
{
    // TODO call vcx_connection_deserialize and pass serializedConnection
    // it would return an error code and an integer connection handle in callback
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] connectionDeserialize:serializedConnection completion:^(NSError *error, NSInteger connectionHandle) {
        if (error != nil && error.code != 0)
        {
            NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while deserializing claim offer: %@ :: %ld", error.domain, (long)error.code];
            NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
            //return @0;
            dispatch_async(dispatch_get_main_queue(), ^{
                successful(NO);
            });
        }else{
            NSLog(@"deserializeConnection was successful!");
            //return @(connectionHandle);
            dispatch_async(dispatch_get_main_queue(), ^{
                successful(YES);
            });
        }
    }];
    });
}

-(void)decryptWalletFile: (NSString *) config
              completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] importWallet: config
       completion:^(NSError *error) {
           if(error != nil && error.code != 0){
               NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while importing wallet: %@ :: %ld", error.domain, (long)error.code];
               //reject(indyErrorCode, [NSString stringWithFormat:@"Error occurred while importing wallet: %@ :: %ld",error.domain, (long)error.code], error);
               NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
               dispatch_async(dispatch_get_main_queue(), ^{
                   successful(NO);
               });
           } else {
               //resolve(@{});
               NSLog(@"decryptWalletFile was successful!");
               dispatch_async(dispatch_get_main_queue(), ^{
                   successful(YES);
               });
           }
       }];
    });
}


-(void)shutdownVcx: (BOOL *) deletePool
{
    //resolve([NSNumber numberWithInt:[[[ConnectMeVcx alloc] init] vcxShutdown: deletePool]]);
    //[NSNumber numberWithInt:[[[ConnectMeVcx alloc] init] vcxShutdown: deletePool]];
    [self setLastShutdownVcx:[[[ConnectMeVcx alloc] init] vcxShutdown: deletePool]];
}

-(int)initNullPay {
  return [[[ConnectMeVcx alloc] init] initNullPay];
}

-(void)credentialCreateWithMsgId: (NSString *) sourceId
                  withConnectionHandle: (VcxHandle) connectionHandle
                  withMessageId: (NSString *) messageId
                      completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] credentialCreateWithMsgid:sourceId
      connectionHandle:connectionHandle
                 msgId:messageId
            completion:^(NSError *error, NSInteger credentialHandle, NSString* credentialOffer) {
                if (error != nil && error.code != 0)
                {
                    NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while creating credential handle: %@ :: %ld", error.domain, (long)error.code];
                    //reject(indyErrorCode, @"Error occurred while creating credential handle", error);
                    NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
                    //return @{};
                    dispatch_async(dispatch_get_main_queue(), ^{
                        successful(NO);
                    });
                } else {
//                    NSDictionary* vcxCredentialCreateResult = @{
//                        @"credential_handle": @(credentialHandle),
//                        @"credential_offer": credentialOffer
//                    };
                    //resolve(vcxCredentialCreateResult);
                    NSLog(@"credentialCreateWithMsgId was successful!");
                    //return vcxCredentialCreateResult;
                    dispatch_async(dispatch_get_main_queue(), ^{
                        successful(YES);
                    });
                }
            }];
    });
}

-(void)serializeClaimOffer: (NSInteger)credentialHandle
                completion:(void (^)(BOOL success))successful
{
    // it would return error code, json string of credential inside callback
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] credentialSerialize:credentialHandle completion:^(NSError *error, NSString *claimOffer) {
        if (error != nil && error.code != 0)
        {
            NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while serializing claim offer: %@ :: %ld", error.domain, (long)error.code];
            //reject(indyErrorCode, @"Error occurred while serializing claim offer", error);
            NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
            //return @"";
            dispatch_async(dispatch_get_main_queue(), ^{
                successful(NO);
            });
        }else{
            //resolve(claimOffer);
            NSLog(@"serializeClaimOffer was successful!");
            //return claimOffer;
            dispatch_async(dispatch_get_main_queue(), ^{
                successful(YES);
            });
        }
    }];
    });
}

-(void)deserializeClaimOffer: (NSString *)serializedCredential
                  completion:(void (^)(BOOL success))successful
{
    // it would return an error code and an integer credential handle in callback
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] credentialDeserialize:serializedCredential
        completion:^(NSError *error, NSInteger credentailHandle) {
            if (error != nil && error.code != 0) {
                NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while deserializing claim offer: %@ :: %ld", error.domain, (long)error.code];
                //reject(indyErrorCode, @"Error occurred while deserializing claim offer", error);
                NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
                //return @0
                dispatch_async(dispatch_get_main_queue(), ^{
                    successful(NO);
                });
            }
            else {
                //resolve(@(credentailHandle));
                NSLog(@"deserializeClaimOffer was successful!");
                //return @(credentailHandle);
                dispatch_async(dispatch_get_main_queue(), ^{
                    successful(YES);
                });
            }
        }];
    });
}

-(void)sendClaimRequest: (NSInteger) credentialHandle
                  withConnectionHandle: (VcxHandle) connectionHandle
                  withPaymentHandle: (vcx_payment_handle_t) paymentHandle
             completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] credentialSendRequest:credentialHandle
      connectionHandle:connectionHandle
         paymentHandle:paymentHandle
            completion:^(NSError *error) {
                if (error != nil && error.code != 0)
                {
                    NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while sending claim request: %@ :: %ld", error.domain, (long)error.code];
                    //reject(indyErrorCode, @"Error occurred while sending claim request", error);
                    NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
                    dispatch_async(dispatch_get_main_queue(), ^{
                        successful(NO);
                    });
                }
                else {
                    //resolve(@{});
                    NSLog(@"sendClaimRequest was successful!");
                    dispatch_async(dispatch_get_main_queue(), ^{
                        successful(YES);
                    });
                }
            }];
    });
}

-(void)initWithConfig: (NSString *)config
{
    // TODO: call vcx_init_with_config of libvcx
    // pass a config as json string
    // callback would get an error code and a json string back in case of success
    NSError *error = nil; // remove this line after integrating libvcx method
    if (error != nil && error.code != 0)
    {
        NSString *indyErrorCode = [NSString stringWithFormat:@"%ld", (long)error.code];
        //reject(indyErrorCode, @"Init failed with error", error);
        NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
    } else {
        //resolve(@{});
        NSLog(@"initWithConfig was successful!");
    }
}


//[[[ConnectMeVcx alloc] init] agentProvisionAsync:config completion:^(NSError *error, NSString *oneTimeInfo) {
//    NSLog(@"applicationDidBecomeActive callback:%@",oneTimeInfo);
//    if (error != nil && error.code != 0)
//    {
//        NSString *indyErrorCode = [NSString stringWithFormat:@"%ld", (long)error.code];
//        reject(indyErrorCode, [NSString stringWithFormat:@"Error occurred while creating one time info: %@ :: %ld",error.domain, (long)error.code], error);
//
//    }else{
//        resolve(oneTimeInfo);
//    }
//}];
-(void)createOneTimeInfo: (NSString *)config
    completion:(void (^)(BOOL success))successful
{
    // pass a config as json string
    // callback would get an error code and a json string back in case of success

    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
        [[[ConnectMeVcx alloc] init] agentProvisionAsync:config completion:^(NSError *error, NSString *oneTimeInfo) {
            NSLog(@"applicationDidBecomeActive callback:%@",oneTimeInfo);
            if (error != nil && error.code != 0)
            {
                NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while creating one time info: %@ :: %ld", error.domain, (long)error.code];
                //reject(indyErrorCode, [NSString stringWithFormat:@"Error occurred while creating one time info: %@ :: %ld",error.domain, (long)error.code], error);
                NSLog(@"Value of indyErrorCode is: %@ -- %@", indyErrorCode, oneTimeInfo);
                [self setLastOneTimeInfoError:error];
                dispatch_async(dispatch_get_main_queue(), ^{
                    successful(NO);
                });
            }else{
                //resolve(oneTimeInfo);
                NSLog(@"createOneTimeInfo was successful: %@", oneTimeInfo);
                [self setLastOneTimeInfo:oneTimeInfo];
                dispatch_async(dispatch_get_main_queue(), ^{
                    successful(YES);
                });
            }
        }];

    });
}

-(void)createConnectionWithInvite: (NSString *)invitationId
                  inviteDetails: (NSString *)inviteDetails
                       completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] connectionCreateWithInvite:invitationId
      inviteDetails:inviteDetails
         completion:^(NSError *error, NSInteger connectionHandle) {
             if (error != nil && error.code != 0)
             {
                 NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while creating connection: %@ :: %ld", error.domain, (long)error.code];
                 //reject(indyErrorCode, @"Error occurred while creating connection", error);
                 NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
                 dispatch_async(dispatch_get_main_queue(), ^{
                     successful(NO);
                 });
             } else {
                 //resolve(@(connectionHandle));
                 NSLog(@"createConnectionWithInvite was successful!");
                 dispatch_async(dispatch_get_main_queue(), ^{
                     successful(YES);
                 });
             }
         }];
    });
}

// TODO:repalce with acceptInvitation when vcx integration done
-(void)vcxAcceptInvitation: (VcxHandle )connectionHandle
                  connectionType: (NSString *)connectionType
                completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] connectionConnect:connectionHandle
        connectionType:connectionType
            completion:^(NSError *error, NSString *inviteDetails) {

                if (error != nil && error.code != 0)
                {
                    NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while accepitng connection: %@ :: %ld", error.domain, (long)error.code];
                    //reject(indyErrorCode, @"Error occurred while accepitng connection", error);
                    NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
                    dispatch_async(dispatch_get_main_queue(), ^{
                        successful(NO);
                    });
                } else {
                    //resolve(inviteDetails);
                    NSLog(@"vcxAcceptInvitation was successful!");
                    dispatch_async(dispatch_get_main_queue(), ^{
                        successful(YES);
                    });
                }
            }];
    });
}

// TODO:repalce with updatePushToken when vcx integration done
-(void)vcxUpdatePushToken: (NSString *)config
    completion:(void (^)(BOOL success))successful
{
    // TODO: call vcx_agent_update_info of libvcx
    // pass a config as json string
    // callback would get an error code
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
        [[[ConnectMeVcx alloc] init] agentUpdateInfo:config completion:^(NSError *error) {
            if (error != nil && error.code != 0)
            {
                NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while updating push token: %@ :: %ld", error.domain, (long)error.code];
                //reject(indyErrorCode, @"Error occurred while updating push token", error);
                NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
                dispatch_async(dispatch_get_main_queue(), ^{
                    successful(NO);
                });
            } else {
                //resolve(@{});
                NSLog(@"vcxUpdatePushToken was successful!");
                dispatch_async(dispatch_get_main_queue(), ^{
                    successful(YES);
                });
            }
        }];
    });
}

-(NSString*)getGenesisPathWithConfig: (NSString *)config
    fileName: (NSString *)fileName
{
    NSError *error;
    NSString *filePath = [[NSSearchPathForDirectoriesInDomains(NSDocumentDirectory, NSUserDomainMask, YES) firstObject] stringByAppendingPathComponent:fileName];
    NSFileManager *fileManager = [NSFileManager defaultManager];
    if (![fileManager fileExistsAtPath: filePath])
    {
        BOOL success=[config writeToFile:filePath atomically:YES encoding:NSUTF8StringEncoding error:&error];
        if(!success)
        {
            //resolve(@"error while creating genesis file");
            NSLog(@"error while creating genesis file");
            return nil;
        }
    }
    //resolve(filePath);
    NSLog(@"getGenesisPathWithConfig was successful: %@", filePath);
    return filePath;
}

-(void)updateClaimOfferState: (int)credentialHandle
                  completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] credentialUpdateState:credentialHandle
        completion:^(NSError *error, NSInteger state)
     {
         if (error != nil && error.code != 0) {
             NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while updating claim offer state: %@ :: %ld", error.domain, (long)error.code];
             //reject(indyErrorCode, @"Error occurred while updating claim offer state", error);
             NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
             dispatch_async(dispatch_get_main_queue(), ^{
                 successful(NO);
             });
         }
         else {
             //resolve(@(state));
             NSLog(@"updateClaimOfferState was successful!");
             dispatch_async(dispatch_get_main_queue(), ^{
                 successful(YES);
             });
         }
     }];
    });
}

-(void)getClaimOfferState: (int)credentialHandle
               completion:(void (^)(BOOL success))successful
{
    // TODO: Add vcx wrapper method for vcx_credential_get_state
    // call vcx_credential_get_state and pass credentialHandle
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] credentialGetState:credentialHandle completion:^(NSError *error, NSInteger state) {
        if (error != nil && error.code != 0) {
            NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while getting claim offer state: %@ :: %ld", error.domain, (long)error.code];
            //reject(indyErrorCode, @"Error occurred while getting claim offer state", error);
            NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
            dispatch_async(dispatch_get_main_queue(), ^{
                successful(NO);
            });
        }
        else {
            //resolve(@(state));
            NSLog(@"getClaimOfferState was successful!");
            dispatch_async(dispatch_get_main_queue(), ^{
                successful(YES);
            });
        }
    }];
    });
}

-(void)getClaimVcx: (int)credentialHandle
        completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] getCredential:credentialHandle completion:^(NSError *error, NSString *credential) {
        if (error != nil && error.code != 0) {
            NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while getting claim: %@ :: %ld", error.domain, (long)error.code];
            //reject(indyErrorCode, @"Error occurred while getting claim", error);
            NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
            dispatch_async(dispatch_get_main_queue(), ^{
                successful(NO);
            });
        }
        else {
            //resolve(credential);
            NSLog(@"getClaimVcx was successful!");
            dispatch_async(dispatch_get_main_queue(), ^{
                successful(YES);
            });
        }
    }];
    });
}

-(void)exportWallet: (NSString *)exportPath
                  encryptWith: (NSString *)encryptionKey
         completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] exportWallet:exportPath
      encryptWith:encryptionKey
       completion:^(NSError *error, NSInteger exportHandle) {
           if (error != nil && error.code != 0)
           {
               NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while exporting wallet: %@ :: %ld", error.domain, (long)error.code];
               //reject(indyErrorCode, @"Error occurred while exporting wallet", error);
               NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
               dispatch_async(dispatch_get_main_queue(), ^{
                   successful(NO);
               });
           } else {
               //resolve(@(exportHandle));
               NSLog(@"exportWallet was successful!");
               dispatch_async(dispatch_get_main_queue(), ^{
                   successful(YES);
               });
           }
       }];
    });
}

-(void)setWalletItem: (NSString *) key
          value: (NSString *) value
          completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
        NSString *recordType = @"record_type";
        [[[ConnectMeVcx alloc] init] addRecordWallet:recordType
            recordId:key
         recordValue:value
          completion:^(NSError *error) {
              if (error != nil && error.code != 0)
              {
                  if ([error.domain isEqualToString:@"VcxErrorDomain"] && error.code == 213 )
                  {
                      //resolve(@-1);
                      NSLog(@"setWalletItem called on duplicate key: %@", key);
                      dispatch_async(dispatch_get_main_queue(), ^{
                          successful(NO);
                      });
                  } else {
                      NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while adding wallet item: %@ :: %ld", error.domain, (long)error.code];
                      //reject(indyErrorCode, [NSString stringWithFormat:@"Error occurred while adding wallet item: %@ :: %ld",error.domain, (long)error.code], error);
                      NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
                      dispatch_async(dispatch_get_main_queue(), ^{
                          successful(NO);
                      });
                  }
              } else {
                  //resolve(@0);
                  NSLog(@"setWalletItem was successful!");
                  dispatch_async(dispatch_get_main_queue(), ^{
                      successful(YES);
                  });
              }
          }];
    });
}

-(void)getWalletItem: (NSString *) key
          completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
        NSString *recordType = @"record_type";
        [[[ConnectMeVcx alloc] init] getRecordWallet:recordType
                                            recordId:key
                                          completion:^(NSError *error, NSString *result)
         {
             if (error != nil && error.code != 0)
             {
                 if ([error.domain isEqualToString:@"VcxErrorDomain"] && error.code == 212 )
                 {
                     //resolve(nil);
                     NSLog(@"getWalletItem could not find key: %@", key);
                     dispatch_async(dispatch_get_main_queue(), ^{
                         successful(NO);
                     });
                 } else {
                     NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while getting wallet item: %@ :: %ld", error.domain, (long)error.code];
                     //reject(indyErrorCode, [NSString stringWithFormat:@"Error occurred while getting wallet item: %@ :: %ld",error.domain, (long)error.code], error);
                     NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
                     dispatch_async(dispatch_get_main_queue(), ^{
                         successful(NO);
                     });
                 }
             } else {
                 //resolve(result);
                 NSLog(@"getWalletItem was successful!");
                 dispatch_async(dispatch_get_main_queue(), ^{
                     successful(YES);
                 });
             }
         }];
    });
}


- (void)deleteWalletItem: (NSString *) key
        completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
        NSString *recordType = @"record_type";
        [[[ConnectMeVcx alloc] init] deleteRecordWallet:recordType
           recordId:key
         completion:^(NSError *error) {
             if (error != nil && error.code != 0)
             {
                 NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while deleting wallet item: %@ :: %ld", error.domain, (long)error.code];
                 //reject(indyErrorCode, [NSString stringWithFormat:@"Error occurred while deleting wallet item: %@ :: %ld",error.domain, (long)error.code], error);
                 NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
                 [self setLastDeleteWasSuccessful:NO];
                 [self setLastDeleteError:error];
                 dispatch_async(dispatch_get_main_queue(), ^{
                     successful(NO);
                 });
             } else {
                 // resolve(@0);
                 NSLog(@"deleteWalletItem was successful!");
                 [self setLastDeleteWasSuccessful:YES];
                 dispatch_async(dispatch_get_main_queue(), ^{
                     successful(YES);
                 });
             }
         }];
    });
}

-(void)updateWalletItem: (NSString *) key
             value: (NSString *) value
             completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    NSString *recordType = @"record_type";

    [[[ConnectMeVcx alloc] init] updateRecordWallet:recordType
       withRecordId:key
    withRecordValue:value
     withCompletion:^(NSError *error) {
         if (error != nil && error.code != 0)
         {
             NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while updating wallet item: %@ :: %ld", error.domain, (long)error.code];
             //reject(indyErrorCode, [NSString stringWithFormat:@"Error occurred while updating wallet item: %@ :: %ld",error.domain, (long)error.code], error);
             NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
             dispatch_async(dispatch_get_main_queue(), ^{
                 successful(NO);
             });
         } else {
             //resolve(@0);
             NSLog(@"updateWalletItem was successful!");
             dispatch_async(dispatch_get_main_queue(), ^{
                 successful(YES);
             });
         }
     }];
    });
}

-(void)downloadMessages: (NSString *) messageStatus
                  uid_s: (NSString *) uid_s
                 pwdids: (NSString *) pwdids
             completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] downloadMessages: messageStatus uid_s:uid_s pwdids:pwdids completion:^(NSError *error, NSString *messages) {


        if (error != nil && error.code !=0) {
            NSString *indyErrorCode = [NSString stringWithFormat:@"%ld", (long)error.code];
            //reject(indyErrorCode, @"Error occured while downloading messages", error);
            NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
            dispatch_async(dispatch_get_main_queue(), ^{
                successful(NO);
            });
        } else{
            //resolve(messages);
            NSLog(@"downloadMessages was successful!");
            dispatch_async(dispatch_get_main_queue(), ^{
                successful(YES);
            });
        }
    }];
    });
}

-(void)proofCreateWithMsgId: (NSString *)sourceId
                  withConnectionHandle: (vcx_connection_handle_t)connectionHandle
                  withMsgId: (NSString *)msgId
                 completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] proofCreateWithMsgId:sourceId
                                 withConnectionHandle:connectionHandle
                                            withMsgId:msgId
                                       withCompletion:^(NSError *error, vcx_proof_handle_t proofHandle, NSString *proofRequest)
     {
         if (error != nil && error.code != 0) {
             NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while downloading proof request: %@ :: %ld", error.domain, (long)error.code];
             //reject(indyErrorCode, @"Error occurred while downloading proof request", error);
             NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
             dispatch_async(dispatch_get_main_queue(), ^{
                 successful(NO);
             });
         }
         else {
            //resolve(@{
            //   @"proofHandle": @(proofHandle),
            //   @"proofRequest": proofRequest
            //});
             NSLog(@"proofCreateWithMsgId was successful!");
             dispatch_async(dispatch_get_main_queue(), ^{
                 successful(YES);
             });
         }
     }];
    });
}

-(void)proofRetrieveCredentials:(vcx_proof_handle_t)proofHandle
                     completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] proofRetrieveCredentials:proofHandle
                                           withCompletion:^(NSError *error, NSString *matchingCredentials)
     {
         if (error != nil && error.code != 0) {
             NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while retrieving matching credentials: %@ :: %ld", error.domain, (long)error.code];
             //reject(indyErrorCode, @"Error occurred while retrieving matching credentials", error);
             NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
             dispatch_async(dispatch_get_main_queue(), ^{
                 successful(NO);
             });
         }
         else {
             //resolve(matchingCredentials);
             NSLog(@"proofRetrieveCredentials was successful!");
             dispatch_async(dispatch_get_main_queue(), ^{
                 successful(YES);
             });
         }
     }];
    });
}

-(void)proofGenerate:(vcx_proof_handle_t)proofHandle
                  withSelectedCredentials:(NSString *)selectedCredentials
                  withSelfAttestedAttrs:(NSString *)selfAttestedAttributes
          completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] proofGenerate:proofHandle
                       withSelectedCredentials:selectedCredentials
                         withSelfAttestedAttrs:selfAttestedAttributes
                                withCompletion:^(NSError *error)
     {
         if (error != nil && error.code != 0) {
             NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while generating proof: %@ :: %ld", error.domain, (long)error.code];
             //reject(indyErrorCode, @"Error occurred while generating proof", error);
             NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
             dispatch_async(dispatch_get_main_queue(), ^{
                 successful(NO);
             });
         }
         else {
             //resolve(@{});
             NSLog(@"proofGenerate was successful!");
             dispatch_async(dispatch_get_main_queue(), ^{
                 successful(YES);
             });
         }
     }];
    });
}

-(void)proofSend:(vcx_proof_handle_t)proof_handle
                  withConnectionHandle:(vcx_connection_handle_t)connection_handle
      completion:(void (^)(BOOL success))successful
{
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
    [[[ConnectMeVcx alloc] init] proofSend:proof_handle
                      withConnectionHandle:connection_handle
                            withCompletion:^(NSError *error)
     {
         if (error != nil && error.code != 0) {
             NSString *indyErrorCode = [NSString stringWithFormat:@"Error occurred while sending proof: %@ :: %ld", error.domain, (long)error.code];
             //reject(indyErrorCode, @"Error occurred while sending proof", error);
             NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
             dispatch_async(dispatch_get_main_queue(), ^{
                 successful(NO);
             });
         }
         else {
             //resolve(@{});
             NSLog(@"proofSend was successful!");
             dispatch_async(dispatch_get_main_queue(), ^{
                 successful(YES);
             });
         }
     }];
    });
}

-(NSString*)createWalletKey: (NSInteger) lengthOfKey
{
    // Generate secure random string
    NSMutableData *data = [NSMutableData dataWithLength:lengthOfKey];
    int result = SecRandomCopyBytes(NULL, lengthOfKey, data.mutableBytes);
    if (result == 0) {
        NSLog(@"createWalletKey was successful!");
        NSString* value = [data base64EncodedStringWithOptions:0];
        //resolve(value);
        return value;
    } else {
        //reject(@"W-001", @"Error occurred while generating wallet key", nil);
        NSString *indyErrorCode = @"W-001: Error occurred while generating wallet key";
        NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
        return nil;
    }
}

-(void)getTokenInfo: (vcx_payment_handle_t)paymentHandle
                    completion:(void (^)(BOOL success))successful
{
    // TODO call vcx_connection_serialize and pass connectionHandle
    // it would return a string
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, kNilOptions), ^{
        [[[ConnectMeVcx alloc] init] getTokenInfo:paymentHandle withCompletion:^(NSError *error, NSString *tokenInfo) {

            if (error != nil && error.code != 0)
            {
                NSString *indyErrorCode = [NSString stringWithFormat:@"%ld", (long)error.code];
                //reject(indyErrorCode, @"Error occurred while getting token info", error);
                NSLog(@"Value of indyErrorCode is: %@", indyErrorCode);
                dispatch_async(dispatch_get_main_queue(), ^{
                    successful(NO);
                });
            } else {
                //resolve(tokenInfo);
                NSLog(@"getTokenInfo was successful!");
                dispatch_async(dispatch_get_main_queue(), ^{
                    successful(YES);
                });
            }
        }];

    });

}

@end
