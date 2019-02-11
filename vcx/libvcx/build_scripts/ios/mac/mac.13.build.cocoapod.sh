#!/bin/sh

set -e
source ./shared.functions.sh

START_DIR=$PWD
WORK_DIR=$START_DIR/../../../../../.macosbuild
mkdir -p $WORK_DIR
WORK_DIR=$(abspath "$WORK_DIR")

VCX_SDK=$START_DIR/../../../../..
VCX_SDK=$(abspath "$VCX_SDK")

COMBINED_LIB=$1
IOS_ARCHS=$2
VCX_VERSION=$3

if [ -z ${IOS_ARCHS} ]; then
    echo "please provide the arch e.g arm, arm64, armv7, x86, or x86_64"
    exit 1
fi

if [ -z ${VCX_VERSION} ]; then
    echo "Please provide a vcx version"
    exit 1
fi

bkpIFS="$IFS"
IFS=',()][' read -r -a archs <<<"${IOS_ARCHS}"
echo "Building vcx.${COMBINED_LIB} wrapper for architectures: ${archs[@]}"    ##Or printf "%s\n" ${array[@]}
IFS="$bkpIFS"
cd $VCX_SDK/vcx/wrappers/ios/vcx
#mv lib/libvcx.a lib/libvcx.a.original

tar -czf ~/IOSBuilds/${COMBINED_LIB}/libvcx.a.${COMBINED_LIB}_${VCX_VERSION}_universal.tar.gz $VCX_SDK/vcx/wrappers/ios/vcx/lib/${COMBINED_LIB}.a
cp -v lib/${COMBINED_LIB}.a lib/libvcx.a
xcodebuild -project vcx.xcodeproj -scheme vcx -configuration Debug CONFIGURATION_BUILD_DIR=. clean

rm -rf vcx.framework.previousbuild
IPHONE_SDK=iphoneos
for arch in ${archs[*]}
do
    rm -rf vcx.framework
    if [ "${arch}" = "i386" ] || [ "${arch}" = "x86_64" ]; then
        # This sdk supports i386 and x86_64
        IPHONE_SDK=iphonesimulator
    elif [ "${arch}" = "armv7" ] || [ "${arch}" = "armv7s" ] || [ "${arch}" = "arm64" ]; then
        # This sdk supports armv7, armv7s, and arm64
        IPHONE_SDK=iphoneos
    fi
    xcodebuild -project vcx.xcodeproj -scheme vcx -configuration Debug -arch ${arch} -sdk ${IPHONE_SDK} CONFIGURATION_BUILD_DIR=. build

    if [ -d "./vcx.framework.previousbuild" ]; then
        lipo -create -output combined.ios.vcx vcx.framework/vcx vcx.framework.previousbuild/vcx
        mv combined.ios.vcx vcx.framework/vcx
        rm -rf vcx.framework.previousbuild
    fi
    cp -rp vcx.framework vcx.framework.previousbuild
done

#export GEM_HOME=${HOME}/.gem
#export PATH=${GEM_HOME}/bin:$PATH
# Test the libvcx.a file if the ${IOS_ARCHS} contains i386 or x86_64
if [[ "${IOS_ARCHS}" == *"i386"* ]] || [[ "${IOS_ARCHS}" == *"x86_64"* ]]; then
    xcodebuild -project vcx.xcodeproj -scheme vcx-demo -sdk iphonesimulator build-for-testing
    ## Need to do:
    ## a) gem install cocoapods -- sudo may be needed
    #if [ -z "$(which pod)" ]; then
    #    gem install cocoapods
    #fi
    ## b) pod setup
    if [ ! -d "${HOME}/.cocoapods/repos/master" ]; then
        pod setup
    fi
    ## c) brew install xctool
    if [ -z "$(which xctool)" ]; then
        brew install xctool
    fi
    xctool -project vcx.xcodeproj -scheme vcx-demo run-tests -sdk iphonesimulator
fi

#mv lib/libvcx.a.original lib/libvcx.a
rm lib/libvcx.a
rm -rf vcx.framework.previousbuild

mkdir -p vcx.framework/lib
# IMPORTANT: DO NOT PUT THE libvcx.a FILE INSIDE THE cocoapod AT ALL!!!!!
#cp -v lib/${COMBINED_LIB}.a vcx.framework/lib/libvcx.a

mkdir -p vcx.framework/Headers
cp -v ConnectMeVcx.h vcx.framework/Headers
cp -v include/libvcx.h vcx.framework/Headers
cp -v vcx/vcx.h vcx.framework/Headers
if [ -d $VCX_SDK/vcx/wrappers/ios/vcx/tmp ]; then
    rm -rf $VCX_SDK/vcx/wrappers/ios/vcx/tmp
fi
mkdir -p $VCX_SDK/vcx/wrappers/ios/vcx/tmp/vcx/
cp -rvp vcx.framework $VCX_SDK/vcx/wrappers/ios/vcx/tmp/vcx/
cd $VCX_SDK/vcx/wrappers/ios/vcx/tmp
cp $WORK_DIR/evernym.vcx-sdk.git.commit.log $VCX_SDK/vcx/wrappers/ios/vcx/tmp/vcx/ || true
cp $WORK_DIR/hyperledger.indy-sdk.git.commit.log $VCX_SDK/vcx/wrappers/ios/vcx/tmp/vcx/ || true

zip -r vcx.${COMBINED_LIB}_${VCX_VERSION}_universal.zip vcx
mkdir -p ~/IOSBuilds/${COMBINED_LIB}
cp -v $VCX_SDK/vcx/wrappers/ios/vcx/tmp/vcx.${COMBINED_LIB}_${VCX_VERSION}_universal.zip ~/IOSBuilds/${COMBINED_LIB}
