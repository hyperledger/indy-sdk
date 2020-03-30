#!/bin/sh

set -e
source ./shared.functions.sh

START_DIR=$PWD
WORK_DIR=$START_DIR/../../../../../.macosbuild
mkdir -p $WORK_DIR
WORK_DIR=$(abspath "$WORK_DIR")

INDY_SDK=$WORK_DIR/vcx-indy-sdk
VCX_SDK=$START_DIR/../../../../..
VCX_SDK=$(abspath "$VCX_SDK")
LIBSSL=$WORK_DIR/OpenSSL-for-iPhone
LIBZMQ=$WORK_DIR/libzmq-ios

cp -v $VCX_SDK/vcx/libvcx/target/universal/release/libvcx.a $VCX_SDK/vcx/wrappers/ios/vcx/lib/libvcx.a.tocombine
cp -v $LIBZMQ/dist/ios/lib/libzmq.a $VCX_SDK/vcx/wrappers/ios/vcx/lib/libzmq.a.tocombine
cp -v $LIBZMQ/libsodium-ios/dist/ios/lib/libsodium.a $VCX_SDK/vcx/wrappers/ios/vcx/lib/libsodium.a.tocombine

# sovtoken and nullpay
# cp -v ${BUILD_CACHE}/libnullpay/${LIBNULLPAY_VERSION}/libnullpay.a $VCX_SDK/vcx/wrappers/ios/vcx/lib/libnullpay.a.tocombine
cp -v ${BUILD_CACHE}/libsovtoken-ios/${LIBSOVTOKEN_VERSION}/libsovtoken/universal/libsovtoken.a ${VCX_SDK}/vcx/wrappers/ios/vcx/lib/libsovtoken.a.tocombine