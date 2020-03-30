#!/bin/bash

source ./shared.functions.sh

START_DIR=$PWD
WORK_DIR=$START_DIR/../../../../../../.macosbuild
mkdir -p $WORK_DIR
WORK_DIR=$(abspath "$WORK_DIR")

NDK_DIR=$WORK_DIR/NDK
INDY_SDK=$WORK_DIR/vcx-indy-sdk
VCX_SDK=$START_DIR/../../../../../..
VCX_SDK=$(abspath "$VCX_SDK")

declare -a archs=(
    "arm" "armv7" "arm-linux-androideabi"
    )
archslen=${#archs[@]}

for (( arch=0; arch<${archslen}; arch=arch+3 ));
do
    export ndk_arch=${archs[$arch]}
    export target_arch=${archs[$arch+1]}
    export cross_compile=${archs[$arch+2]}
    LIB_FOLDER="lib"
    if [ "$ndk_arch" == "x86_64" ]; then
        LIB_FOLDER="lib64"
    fi

    cd $VCX_SDK/vcx/wrappers/java/vcx/src/main/jniLibs/${target_arch}
    rm ./libvcx.so
    $NDK_DIR/${ndk_arch}/bin/${cross_compile}-clang -v -shared -o libvcx.so -Wl,--whole-archive \
    libindy.a \
    libvcx.a \
    libzmq.a \
    libsodium.a \
    $NDK_DIR/${ndk_arch}/${cross_compile}/${LIB_FOLDER}/libc++_shared.so \
    $NDK_DIR/${ndk_arch}/sysroot/usr/${LIB_FOLDER}/libz.so \
    $NDK_DIR/${ndk_arch}/sysroot/usr/${LIB_FOLDER}/libm.a \
    $NDK_DIR/${ndk_arch}/sysroot/usr/${LIB_FOLDER}/liblog.so \
    -Wl,--no-whole-archive -z muldefs
    echo "Created $VCX_SDK/vcx/wrappers/java/vcx/src/main/jniLibs/${target_arch}/libvcx.so"
    cd $VCX_SDK/vcx/wrappers/java/vcx/src/main/jniLibs
    rm -rf armeabi-v7a
    mv armv7 armeabi-v7a
    # rm libvcxall_${target_arch}.zip
    cd $VCX_SDK/vcx/wrappers/java/vcx/
    ./gradlew clean assembleDebug
    cd $VCX_SDK/vcx/wrappers/java/vcx/build/outputs/aar/
    # install generated .aar file
    find . -name "*-debug.aar" -exec mvn install:install-file -Dfile={} -DgroupId=com.connectme \
    -DartifactId=vcx -Dversion=$(ls | cut -c 17-38) -Dpackaging=aar \;
    # copy version number to clipboard
    ls | cut -c 17-38 | pbcopy
    # KS: Commenting it out because we don't need zip for local development
    # zip -r libvcxall_${target_arch}.zip ${target_arch}
    # echo "Created $VCX_SDK/vcx/wrappers/java/vcx/src/main/jniLibs/libvcxall_${target_arch}.zip"
done
