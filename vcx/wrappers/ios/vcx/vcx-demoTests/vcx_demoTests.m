//
//  vcx_demoTests.m
//  vcx-demoTests
//
//  Created by yaswanthsvist on 4/30/18.
//  Copyright © 2018 GuestUser. All rights reserved.
//

#import <XCTest/XCTest.h>
#import "RNIndyTests.h"
#import "IndyCallbacks.h"

@interface vcx_demoTests : XCTestCase

@end

@implementation vcx_demoTests

- (void)setUp {
    [super setUp];
    // Put setup code here. This method is called before the invocation of each test method in the class.
}

- (void)tearDown {
    // Put teardown code here. This method is called after the invocation of each test method in the class.
    [super tearDown];
}

- (void)testStartFreshAndGeneratePassphrase {
    // This is an example of a functional test case.
    // Use XCTAssert and related functions to verify your tests produce the correct results.
    XCTestExpectation *expectation = [self expectationWithDescription:@"startFreshAndGeneratePassphrase timed out!"];

    RNIndy *indy = [[RNIndy alloc] init];
    VcxLogger *vcxLogger = [[VcxLogger alloc] init];
    IndyCallbacks *indyCallbacks = [[IndyCallbacks alloc] init];

    [RNIndyTests startFreshAndGeneratePassphrase:indy completion:^(BOOL success) {
        NSLog(@"TEST startFreshAndGeneratePassphrase %@!", success ? @"succeeded" : @"failed");
        XCTAssertTrue(success);
        [expectation fulfill];
    }];

    [self waitForExpectationsWithTimeout:120.0 handler:^(NSError *error) {
        if (error) {
            NSLog(@"Error: %@", error);
        }
    }];
}

- (void)testPerformanceExample {
    // This is an example of a performance test case.
    [self measureBlock:^{
        // Put the code you want to measure the time of here.
    }];
}

@end
