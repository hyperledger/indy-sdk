//
//  Base58Utils.h
//  Indy-demo
//
//  Created by Anastasia Tarasova on 12/06/2019.
//  Copyright © 2019 Hyperledger. All rights reserved.
//

#import <Foundation/Foundation.h>
#import <Indy_demo-Swift.h>

@class Base58;

@interface Base58Utils : NSObject 

+ (NSData *) decode:(NSString *) str;
+ (NSString *) encodeData:(NSData *) data;

@end
