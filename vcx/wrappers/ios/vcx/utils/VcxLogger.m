//
//  VcxLogger.m
//  vcx
//
//  Created by Evernym on 12/13/18.
//  Copyright © 2018 GuestUser. All rights reserved.
//

#import "VcxLogger.h"
#import "NSError+VcxError.h"
#import "VcxTypes.h"
#import "VcxErrors.h"
#include "vcx.h"

@interface VcxLogger ()

@property(strong, readwrite) NSMutableArray *callbacks;

@end

@implementation VcxLogger : NSObject

+ (void)setDefaultLogger:(NSString *)pattern {
    vcx_set_default_logger([pattern UTF8String]);
}

+ (void)setLogger:(id)logCb {
    [VcxLogger sharedInstance].callbacks[0] = [logCb copy];
    vcx_set_logger(nil, nil, logCallback, nil);
}

+ (VcxLogger *)sharedInstance {
    static VcxLogger *instance = nil;
    static dispatch_once_t dispatch_once_block;

    dispatch_once(&dispatch_once_block, ^{
        instance = [VcxLogger new];
    });

    return instance;
}

- (VcxLogger *)init {
    self = [super init];
    if (self) {
        self.callbacks = [[NSMutableArray alloc] init];
    }
    return self;
}

void logCallback(const void *context,
        uint32_t level,
        const char *target,
        const char *message,
        const char *modulePath,
        const char *file,
        uint32_t line) {
    id block = [VcxLogger sharedInstance].callbacks[0];

    void (^completion)(NSObject *, NSNumber *, NSString *, NSString *, NSString *, NSString *, NSNumber *) =
    (void (^)(NSObject *context, NSNumber *level, NSString *target, NSString *message, NSString *modulePath, NSString *file, NSNumber *line)) block;
    NSObject *sarg0 = (__bridge NSObject *) context;
    NSNumber *sarg1 = @(level);
    NSString *sarg2 = nil;
    if (target) {
        sarg2 = [NSString stringWithUTF8String:target];
    }
    NSString *sarg3 = nil;
    if (message) {
        sarg3 = [NSString stringWithUTF8String:message];
    }
    NSString *sarg4 = nil;
    if (modulePath) {
        sarg4 = [NSString stringWithUTF8String:modulePath];
    }
    NSString *sarg5 = nil;
    if (file) {
        sarg5 = [NSString stringWithUTF8String:file];
    }
    NSNumber *sarg6 = @(line);

    if (completion) {
        completion(sarg0, sarg1, sarg2, sarg3, sarg4, sarg5, sarg6);
    }
}

@end

