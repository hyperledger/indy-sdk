#import <Foundation/Foundation.h>
#import <XCTest/XCTest.h>
#import "PoolUtils.h"
#import "TestUtils.h"
#import "Base58Utils.h"

@interface DidHignCases : XCTestCase

@end

@implementation DidHignCases {
    IndyHandle walletHandle;
    NSError *ret;
}


- (void)setUp {
    [super setUp];
    [TestUtils cleanupStorage];

    ret = [[PoolUtils sharedInstance] setProtocolVersion:[TestUtils protocolVersion]];
    XCTAssertEqual(ret.code, Success, @"PoolUtils::setProtocolVersion() failed!");

    ret = [[WalletUtils sharedInstance] createAndOpenWalletWithHandle:&walletHandle];
    // Put setup code here. This method is called before the invocation of each test method in the class.
}

- (void)tearDown {
    // Put teardown code here. This method is called after the invocation of each test method in the class.
    [[WalletUtils sharedInstance] closeWalletWithHandle:walletHandle];
    [TestUtils cleanupStorage];
    [super tearDown];
}

// MARK: - Create my did

- (void)testCreateMyDidWorksForEmptyJson {
    // 1. Obtain my did
    NSString *myDid;
    NSString *myVerKey;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&myDid
                                                     outMyVerkey:&myVerKey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");
    XCTAssertEqual([[Base58Utils decode:myDid] length], 16, @"length of myDid != 16");
    XCTAssertEqual([[Base58Utils decode:myVerKey] length], 32, @"length of myVerKey != 32");
}

- (void)testCreateMyDidWorksWithSeed {
    // 1. Obtain my did
    NSString *myDid;
    NSString *myVerKey;
    ret = [[DidUtils sharedInstance] createAndStoreMyDidWithWalletHandle:walletHandle
                                                                    seed:[TestUtils mySeed1]
                                                                outMyDid:&myDid
                                                             outMyVerkey:&myVerKey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");
    XCTAssertTrue([myDid isEqualToString:[TestUtils myDid1]], @"wrong myDid!");
    XCTAssertTrue([myVerKey isEqualToString:[TestUtils myVerkey1]], @"wrong myVerKey!");
}

// MARK: - Replace keys Start

- (void)testReplaceKeysStartWorks {
    // 1. create my did
    NSString *myDid;
    NSString *myVerkey;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&myDid
                                                     outMyVerkey:&myVerkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 2. replace keys
    NSString *newVerkey;
    ret = [[DidUtils sharedInstance] replaceKeysStartForDid:myDid
                                               identityJson:@"{}"
                                               walletHandle:walletHandle
                                                outMyVerKey:&newVerkey];
    XCTAssertFalse([myVerkey isEqualToString:newVerkey], @"verkey is the same!");
}

- (void)testReplaceKeysStartWorksForNotExistingDid {
    // 1. replace keys start
    ret = [[DidUtils sharedInstance] replaceKeysStartForDid:[TestUtils trusteeDid]
                                               identityJson:@"{}"
                                               walletHandle:walletHandle
                                                outMyVerKey:nil];
    XCTAssertEqual(ret.code, WalletItemNotFound, @"DidUtils:replaceKeysStartForDid returned wrong code.");
}

// MARK: - Replace keys apply

- (void)testReplaceKeysApplyWorks {
    // 1. create my did
    NSString *myDid;
    NSString *myVerkey;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&myDid
                                                     outMyVerkey:&myVerkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 2. Replace keys start
    NSString *newVerkey;
    ret = [[DidUtils sharedInstance] replaceKeysStartForDid:myDid
                                               identityJson:@"{}"
                                               walletHandle:walletHandle
                                                outMyVerKey:&newVerkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::replaceKeysStartForDid() failed");

    // 3. Replace keys apply
    ret = [[DidUtils sharedInstance] replaceKeysApplyForDid:myDid
                                               walletHandle:walletHandle];
    XCTAssertEqual(ret.code, Success, @"DidUtils::replaceKeysApplyForDid() failed");
}

- (void)testReplaceKeysApplyWorksWithoutCallingReplaceStart {
    // 2. create my did
    NSString *myDid;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&myDid
                                                     outMyVerkey:nil];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 2. Replace keys apply
    ret = [[DidUtils sharedInstance] replaceKeysApplyForDid:myDid
                                               walletHandle:walletHandle];
    XCTAssertEqual(ret.code, WalletItemNotFound, @"DidUtils::replaceKeysApplyForDid() returned wrong error code.");
}

// MARK: - Replace key

- (void)testReplaceKeysDemo {
    // 1. create and open pool
    IndyHandle poolHandle = 0;
    ret = [[PoolUtils sharedInstance] createAndOpenPoolLedgerWithPoolName:[TestUtils pool]
                                                               poolHandle:&poolHandle];
    XCTAssertEqual(ret.code, Success, @"PoolUtils::createAndOpenPoolLedgerWithPoolName() failed");

    // 2. get my did
    NSString *myDid;
    NSString *myVerkey;
    [[DidUtils sharedInstance] createAndStoreAndPublishDidWithWalletHandle:walletHandle
                                                                poolHandle:poolHandle
                                                                       did:&myDid
                                                                    verkey:&myVerkey];

    // 3. start replacing of keys
    NSString *newVerkey;
    ret = [[DidUtils sharedInstance] replaceKeysStartForDid:myDid
                                               identityJson:@"{}"
                                               walletHandle:walletHandle
                                                outMyVerKey:&newVerkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::replaceKeysStartForDid() failed");

    // 4. Send nym request to ledger with new verkey
    NSString *nymRequest;
    ret = [[LedgerUtils sharedInstance] buildNymRequestWithSubmitterDid:myDid
                                                              targetDid:myDid
                                                                 verkey:newVerkey
                                                                  alias:nil
                                                                   role:nil
                                                             outRequest:&nymRequest];
    XCTAssertEqual(ret.code, Success, @"LedgerUtils::buildNymRequestWithSubmitterDid() failed");

    NSString *nymResponce;
    ret = [[LedgerUtils sharedInstance] signAndSubmitRequestWithPoolHandle:poolHandle
                                                              walletHandle:walletHandle
                                                              submitterDid:myDid
                                                               requestJson:nymRequest
                                                           outResponseJson:&nymResponce];
    XCTAssertEqual(ret.code, Success, @"LedgerUtils::signAndSubmitRequestWithPoolHandle() failed");

    // 5. Send schema request before applying replacing of keys
    NSString *schemaRequest;
    ret = [[LedgerUtils sharedInstance] buildSchemaRequestWithSubmitterDid:myDid
                                                                      data:[TestUtils gvtSchema]
                                                                resultJson:&schemaRequest];
    XCTAssertEqual(ret.code, Success, @"LedgerUtils::buildSchemaRequestWithSubmitterDid() failed");

    NSString *schemaResponse = nil;
    ret = [[LedgerUtils sharedInstance] signAndSubmitRequestWithPoolHandle:poolHandle
                                                              walletHandle:walletHandle
                                                              submitterDid:myDid
                                                               requestJson:schemaRequest
                                                           outResponseJson:&schemaResponse];
    XCTAssertEqual(ret.code, Success, @"LedgerUtils::signAndSubmitRequestWithPoolHandle() returned not Success");

    NSDictionary *response = [NSDictionary fromString:schemaResponse];
    XCTAssertTrue([response[@"op"] isEqualToString:@"REQNACK"], @"wrong response type");

    // 6. Apply replacing of keys
    ret = [[DidUtils sharedInstance] replaceKeysApplyForDid:myDid
                                               walletHandle:walletHandle];
    XCTAssertEqual(ret.code, Success, @"DidUtils::replaceKeysApplyForDid() failed");

    // 7. Send schema request.
    ret = [[LedgerUtils sharedInstance] signAndSubmitRequestWithPoolHandle:poolHandle
                                                              walletHandle:walletHandle
                                                              submitterDid:myDid
                                                               requestJson:schemaRequest
                                                           outResponseJson:nil];
    XCTAssertEqual(ret.code, Success, @"LedgerUtils::signAndSubmitRequestWithPoolHandle() failed");

    [[PoolUtils sharedInstance] closeHandle:poolHandle];
}


// MARK: - Store their did

- (void)testStoreTheidDidWorks {
    // 1. Store their did
    ret = [[DidUtils sharedInstance] storeTheirDidFromPartsWithWalletHandle:walletHandle
                                                                   theirDid:[TestUtils trusteeDid]
                                                                theirVerkey:[TestUtils trusteeVerkey]];
    XCTAssertEqual(ret.code, Success, @"DidUtils:storeTheirDid failed");
}

// MARK: - Key for Did

- (void)testKeyForDidWorksForMyDid {
    // 1. Create did
    NSString *did;
    NSString *verkey;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&did
                                                     outMyVerkey:&verkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 2. Get key for did
    NSString *receivedVerkey;
    ret = [[DidUtils sharedInstance] keyForDid:did
                                    poolHandle:-1
                                  walletHandle:walletHandle
                                           key:&receivedVerkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::keyForDid() failed");
    XCTAssertTrue([verkey isEqualToString:receivedVerkey], @"Keys are not equal");
}

// MARK: - Key for local Did

- (void)testKeyForLocalDidWorksForMyDid {
    // 1. Create did
    NSString *did;
    NSString *verkey;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&did
                                                     outMyVerkey:&verkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 2. Get key for local did
    NSString *receivedVerkey;
    ret = [[DidUtils sharedInstance] keyForLocalDid:did
                                       walletHandle:walletHandle
                                                key:&receivedVerkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::keyForDid() failed");
    XCTAssertTrue([verkey isEqualToString:receivedVerkey], @"Keys are not equal");
}

// MARK: - Set endpoint for Did

- (void)testSetEndpointForDid {
    // 1. Create did
    NSString *did;
    NSString *verkey;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&did
                                                     outMyVerkey:&verkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 2. Set endpoint for did
    ret = [[DidUtils sharedInstance] setEndpointAddress:[TestUtils endpoint]
                                           transportKey:verkey
                                                 forDid:did
                                           walletHandle:walletHandle];
    XCTAssertEqual(ret.code, Success, @"DidUtils::keyForDid() failed");
}

// MARK: - Get endpoint for Did

- (void)testGetEndpointForDid {
    // 1. Create did
    NSString *did;
    NSString *verkey;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&did
                                                     outMyVerkey:&verkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 2. Set endpoint for did
    ret = [[DidUtils sharedInstance] setEndpointAddress:[TestUtils endpoint]
                                           transportKey:verkey
                                                 forDid:did
                                           walletHandle:walletHandle];
    XCTAssertEqual(ret.code, Success, @"DidUtils::keyForDid() failed");

    NSString *endpoint;
    NSString *transportKey;
    ret = [[DidUtils sharedInstance] getEndpointForDid:did
                                          walletHandle:walletHandle
                                            poolHandle:-1
                                               address:&endpoint
                                          transportKey:&transportKey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::getEndpointForDid() failed");
    XCTAssertTrue([endpoint isEqualToString:[TestUtils endpoint]], @"Endpoints are not equal");
    XCTAssertTrue([transportKey isEqualToString:verkey], @"Keys are not equal");
}

// MARK: - Set Did metadata

- (void)testSetDidMetadata {
    // 1. Create did
    NSString *did;
    NSString *verkey;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&did
                                                     outMyVerkey:&verkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 3. Set did metadata
    ret = [[DidUtils sharedInstance] setMetadata:[TestUtils someMetadata]
                                          forDid:did
                                    walletHandle:walletHandle];
    XCTAssertEqual(ret.code, Success, @"DidUtils::setMetadata() failed");
}

// MARK: - Get Did metadata

- (void)testGetDidMetadata {
    // 1. Create did
    NSString *did;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&did
                                                     outMyVerkey:nil];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 2. Set did metadata
    ret = [[DidUtils sharedInstance] setMetadata:[TestUtils someMetadata]
                                          forDid:did
                                    walletHandle:walletHandle];
    XCTAssertEqual(ret.code, Success, @"DidUtils::setMetadata() failed");

    // 3. Get did metadata
    NSString *metadata;
    ret = [[DidUtils sharedInstance] getMetadataForDid:did
                                          walletHandle:walletHandle
                                              metadata:&metadata];
    XCTAssertEqual(ret.code, Success, @"DidUtils::getMetadataForDid() failed");
    XCTAssertTrue([metadata isEqualToString:[TestUtils someMetadata]], @"Metadata are not equal");
}

// MARK: - Abbreviate verkey

- (void)testAbbreviateVerkeyForAbbreviatedKey {
    // 1. Create did
    NSString *did;
    NSString *verkey;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&did
                                                     outMyVerkey:&verkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 2. Abbreviate verkey
    NSString *abbrVerkey;
    ret = [[DidUtils sharedInstance] abbreviateVerkey:did
                                           fullVerkey:verkey
                                               verkey:&abbrVerkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::abbreviateVerkey() failed");
    XCTAssertFalse([verkey isEqualToString:abbrVerkey], @"Keys are equal");
}

// MARK: - Qualify did

- (void)testQualifyDid {
    // 1. Create did
    NSString *did;
    NSString *verkey;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&did
                                                     outMyVerkey:&verkey];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 2. Qualify did
    NSString *method = @"peer";
    NSString *fullQualifiedDid;
    ret = [[DidUtils sharedInstance] qualifyDid:did
                                         method:method
                                   walletHandle:walletHandle
                               fullQualifiedDid:&fullQualifiedDid];
    XCTAssertEqual(ret.code, Success, @"DidUtils::abbreviateVerkey() failed");
    NSString *expectedDid = [NSString stringWithFormat:@"did:%@:%@", method, did];
    XCTAssertTrue([fullQualifiedDid isEqualToString:expectedDid], @"Did are equal");
}

// MARK: - List DIDs

- (void)testListDids {
    // 1. Create did1
    NSString *did1;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&did1
                                                     outMyVerkey:nil];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 2. Create did2
    NSString *did2;
    ret = [[DidUtils sharedInstance] createMyDidWithWalletHandle:walletHandle
                                                       myDidJson:@"{}"
                                                        outMyDid:&did2
                                                     outMyVerkey:nil];
    XCTAssertEqual(ret.code, Success, @"DidUtils::createMyDidWithWalletHandle() failed");

    // 3. List DIDs in wallet
    NSString *metadata;
    ret = [[DidUtils sharedInstance] listMyDidsWithMeta:walletHandle metadata:&metadata];
    XCTAssertEqual(ret.code, Success, @"DidUtils::listMyDidsWithMeta() failed");
    XCTAssertTrue([metadata containsString:did1], @"Metadata does not contain first DID");
    XCTAssertTrue([metadata containsString:did2], @"Metadata does not contain second DID");
}

@end
