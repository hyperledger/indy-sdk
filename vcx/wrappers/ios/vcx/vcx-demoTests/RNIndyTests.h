//
//  RNIndyTests.h
//  testlibvcx
//
//  Created by Norman Jarvis on 7/25/18.
//  Copyright © 2018 Norman Jarvis. All rights reserved.
//

#import <Foundation/Foundation.h>
#import "RNIndy.h"

@interface RNIndyTests : NSObject

//+(void)decryptWallet:(RNIndy *)indy
//    completion:(void (^)(BOOL))successful;
//+(void)initAndUpdatePushToken:(RNIndy*)indy
//    completion:(void (^)(BOOL success))successful;
//+(void)startFreshAndExportWallet1:(RNIndy*)indy
//    completion:(void (^) (BOOL success))successful;
//+(void)startFreshAndExportWallet2:(RNIndy *)indy
//    completion:(void (^)(BOOL))successful;
//+(void)importBlankWallet1:(RNIndy *)indy
//    completion:(void (^)(BOOL))successful;
//+(void)startFreshAndExportWallet3:(RNIndy*)indy
//        completion:(void (^)(BOOL success))successful;

+(void)startFreshAndGeneratePassphrase:(RNIndy*)indy
    completion:(void (^)(BOOL success))successful;

@end