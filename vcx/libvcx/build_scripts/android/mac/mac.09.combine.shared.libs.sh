#!/bin/bash

source ./shared.functions.sh

START_DIR=$PWD
WORK_DIR=$START_DIR/../../../../../.macosbuild
mkdir -p $WORK_DIR
WORK_DIR=$(abspath "$WORK_DIR")

NDK_DIR=$WORK_DIR/NDK
INDY_SDK=$WORK_DIR/vcx-indy-sdk
VCX_SDK=$START_DIR/../../../../..
VCX_SDK=$(abspath "$VCX_SDK")

# declare -a archs=(
#     "arm" "arm" "arm-linux-androideabi" "armeabi"
#     "arm" "armv7" "arm-linux-androideabi" "armeabi-v7a"
#     "arm64" "arm64" "aarch64-linux-android" "arm64-v8a"
#     "x86" "x86" "i686-linux-android" "x86"
#     "x86_64" "x86_64" "x86_64-linux-android" "x86_64"
#     )
# For now, we need only two architectures
declare -a archs=(
    "arm" "armv7" "arm-linux-androideabi" "armeabi-v7a"
    "x86" "x86" "i686-linux-android" "x86"
    )
archslen=${#archs[@]}

for (( arch=0; arch<${archslen}; arch=arch+4 ));
do
    export ndk_arch=${archs[$arch]}
    export target_arch=${archs[$arch+1]}
    export cross_compile=${archs[$arch+2]}
    export aar_arch=${archs[$arch+3]}
    LIB_FOLDER="lib"
    if [ "$ndk_arch" == "x86_64" ]; then
        LIB_FOLDER="lib64"
    fi
    LIB_STL=$NDK_DIR/${ndk_arch}/${cross_compile}/${LIB_FOLDER}/libc++_shared.so
    if [ "$ndk_arch" == "arm" ]; then
        LIB_STL=$NDK_DIR/${ndk_arch}/${cross_compile}/${LIB_FOLDER}/armv7-a/libc++_shared.so
    fi

    cd $VCX_SDK/vcx/wrappers/java/vcx/src/main/jniLibs/${target_arch}
    rm ./libvcx.so
    $NDK_DIR/${ndk_arch}/bin/${cross_compile}-clang -v -shared -o libvcx.so -Wl,--whole-archive \
    libvcx.a \
    libzmq.a \
    libsodium.a \
    libssl.a \
    libcrypto.a \
    ${LIB_STL} \
    $NDK_DIR/${ndk_arch}/sysroot/usr/${LIB_FOLDER}/libz.so \
    $NDK_DIR/${ndk_arch}/sysroot/usr/${LIB_FOLDER}/libm.a \
    $NDK_DIR/${ndk_arch}/sysroot/usr/${LIB_FOLDER}/liblog.so \
    -Wl,--no-whole-archive -z muldefs
    echo "Created $VCX_SDK/vcx/wrappers/java/vcx/src/main/jniLibs/${target_arch}/libvcx.so"
    #cd $VCX_SDK/vcx/wrappers/java/vcx/src/main/jniLibs
    #rm libvcxall_${target_arch}.zip
    # zip -r libvcxall_${target_arch}.zip ${target_arch}
    # echo "Created $VCX_SDK/vcx/wrappers/java/vcx/src/main/jniLibs/libvcxall_${target_arch}.zip"
    if [ "${aar_arch}" != "${target_arch}" ]; then
        cd $VCX_SDK/vcx/wrappers/java/vcx/src/main/jniLibs
        rm -rf ${aar_arch}
        mv ${target_arch} ${aar_arch}
    fi
done
