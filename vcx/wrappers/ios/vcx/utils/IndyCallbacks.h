//
//  IndyCallbacks.h
//  libindy
//
//  Created by Norman Jarvis on 2/18/19.
//  Copyright © 2019 GuestUser. All rights reserved.
//

#import <Foundation/Foundation.h>
#import "indy_mod.h"
#import "indy_types.h"
#import "IndyTypes.h"


#ifndef IndyCallbacks_h
#define IndyCallbacks_h

extern void IndyWrapperCommonCallback(indy_handle_t xcommand_handle,
                                      indy_error_t err);

extern void IndyWrapperCommonHandleCallback(indy_handle_t xcommand_handle,
                                            indy_error_t err,
                                            indy_handle_t pool_handle);

extern void IndyWrapperCommonStringCallback(indy_handle_t xcommand_handle,
                                            indy_error_t err,
                                            const char *const arg1);

extern void IndyWrapperCommonBoolCallback(indy_handle_t xcommand_handle,
                                          indy_error_t err,
                                          indy_bool_t arg1);

extern void IndyWrapperCommonStringStringCallback(indy_handle_t xcommand_handle,
                                                  indy_error_t err,
                                                  const char *const arg1,
                                                  const char *const arg2);

extern void IndyWrapperCommonStringOptStringCallback(indy_handle_t xcommand_handle,
                                                     indy_error_t err,
                                                     const char *const arg1,
                                                     const char *const arg2);

extern void IndyWrapperCommonDataCallback(indy_handle_t xcommand_handle,
                                          indy_error_t err,
                                          const uint8_t *const arg1,
                                          uint32_t arg2);

extern void IndyWrapperCommonStringStringStringCallback(indy_handle_t xcommand_handle,
                                                        indy_error_t err,
                                                        const char *const arg1,
                                                        const char *const arg2,
                                                        const char *const arg3);

extern void IndyWrapperCommonStringDataCallback(indy_handle_t xcommand_handle,
                                                indy_error_t err,
                                                const char *const arg1,
                                                const uint8_t *const arg2,
                                                uint32_t arg3);

extern void IndyWrapperCommonNumberCallback(indy_handle_t xcommand_handle,
                                            indy_error_t err,
                                            indy_i32_t handle);

extern void IndyWrapperCommonHandleNumberCallback(indy_handle_t xcommand_handle,
                                                  indy_error_t err,
                                                  indy_i32_t handle,
                                                  uint32_t count);

extern void IndyWrapperCommonStringOptStringOptStringCallback(indy_handle_t xcommand_handle,
                                                              indy_error_t err,
                                                              const char *const arg1,
                                                              const char *const arg2,
                                                              const char *const arg3);

void IndyWrapperCommonStringStringLongCallback(indy_handle_t xcommand_handle,
                                               indy_error_t err,
                                               const char *arg1,
                                               const char *arg2,
                                               unsigned long long arg3);

@interface IndyCallbacks : NSObject

// MARK: - Store callback and create command handle
- (indy_handle_t)createCommandHandleFor:(id)callback;

- (void)deleteCommandHandleFor:(indy_handle_t)handle;

- (void)complete:(void (^)(NSError *))completion
       forHandle:(indy_handle_t)handle
         ifError:(indy_error_t)ret;

- (void)completeBool:(void (^)(NSError *, BOOL))completion
           forHandle:(indy_handle_t)handle
             ifError:(indy_error_t)ret;

- (void)completeStr:(void (^)(NSError *, NSString *))completion
          forHandle:(indy_handle_t)handle
            ifError:(indy_error_t)ret;

- (void)complete2Str:(void (^)(NSError *, NSString *, NSString *))completion
           forHandle:(indy_handle_t)handle
             ifError:(indy_error_t)ret;

- (void)completeData:(void (^)(NSError *, NSData *))completion
           forHandle:(indy_handle_t)handle
             ifError:(indy_error_t)ret;


- (void)completeStringAndData:(void (^)(NSError *, NSString *, NSData *))completion
                    forHandle:(indy_handle_t)handle
                      ifError:(indy_error_t)ret;

+ (IndyCallbacks *)sharedInstance;

@end

#endif /* IndyCallbacks_h */
