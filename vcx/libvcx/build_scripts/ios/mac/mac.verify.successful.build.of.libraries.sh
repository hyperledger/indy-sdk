#!/bin/sh

source ./shared.functions.sh

START_DIR=$PWD
WORK_DIR=$START_DIR/../../../../../.macosbuild
mkdir -p $WORK_DIR
WORK_DIR=$(abspath "$WORK_DIR")

VCX_SDK=$START_DIR/../../../../..
VCX_SDK=$(abspath "$VCX_SDK")

# BUILD_UNDERWAY=$(sudo launchctl list|grep local.build_libvcx|awk '{print $1}')

# if [ "$BUILD_UNDERWAY" != "-" ]; then
#     echo "The iOS build is currently running ($BUILD_UNDERWAY)! Please wait for it to finish before trying to verify whether or not the build was successful."
#     echo "The output from this script will not reflect the correct status of the full build!"
# fi

# Verify that libindy, and libvcx built correctly for iOS...
cd $START_DIR
grep "error:" ./mac.03.libindy.build.sh.out
echo "-----------------------------------------------------------------------------------------------------------------------------------------------"
cd $WORK_DIR/vcx-indy-sdk/libindy/target
ls -al `find . -name "*.a"`
echo "-----------------------------------------------------------------------------------------------------------------------------------------------"
cd $WORK_DIR/vcx-indy-sdk/libnullpay/target
ls -al `find . -name "*.a"`
echo "-----------------------------------------------------------------------------------------------------------------------------------------------"
cd $START_DIR
grep "error:" ./mac.06.libvcx.build.sh.out
echo "-----------------------------------------------------------------------------------------------------------------------------------------------"
cd $VCX_SDK/vcx/libvcx/target
ls -al `find . -name "libvcx.*"`
echo "-----------------------------------------------------------------------------------------------------------------------------------------------"
cd $VCX_SDK/vcx/wrappers/ios/vcx/lib
ls -alh
echo "-----------------------------------------------------------------------------------------------------------------------------------------------"
cd $START_DIR
grep "error:" ./mac.13.build.cocoapod.sh.out
