//
//  Base58Utils.m
//  Indy-demoTests
//
//  Created by Anastasia Tarasova on 12/06/2019.
//  Copyright © 2019 Hyperledger. All rights reserved.
//

#import "Base58Utils.h"

@interface Base58Utils ()
@end

@implementation Base58Utils

+ (NSData *)decode:(NSString *) str
{
    return [[Base58 class] decode: str];
}

+ (NSString *)encodeData:(NSData *) data
{
    return [[Base58 class] encode: data];
}
@end
