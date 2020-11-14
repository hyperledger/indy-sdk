//
//  Base58Test.m
//  Indy-demo
//
//  Created by Anastasia Tarasova on 23.06.17.
//  Copyright © 2017 Kirill Neznamov. All rights reserved.
//

#import <Foundation/Foundation.h>

#import <XCTest/XCTest.h>
#import "PoolUtils.h"
#import "TestUtils.h"
#import "WalletUtils.h"
#import <Indy/Indy.h>
#import "NSDictionary+JSON.h"
#import "Base58Utils.h"

@interface Base58Test : XCTestCase

@end

@implementation Base58Test

- (void)testDecode
{
    XCTAssertTrue([[Base58Utils decode:@""] isEqualToData:[@"" dataUsingEncoding:NSUTF8StringEncoding allowLossyConversion:false]], @"");
    XCTAssertTrue([[Base58Utils decode:@"3mJr7AoUXx2Wqd"] isEqualToData:[@"1234598760" dataUsingEncoding:NSUTF8StringEncoding allowLossyConversion:false]], @"");
    XCTAssertTrue([[Base58Utils decode:@"3yxU3u1igY8WkgtjK92fbJQCd4BZiiT1v25f"] isEqualToData:[@"abcdefghijklmnopqrstuvwxyz" dataUsingEncoding:NSUTF8StringEncoding allowLossyConversion:false]], @"");
}

@end
