#!/usr/bin/env bash
set -x


if [ -z "${ANDROID_BUILD_FOLDER}" ]; then
    echo STDERR "ANDROID_BUILD_FOLDER is not set. Please set it in the caller script"
    echo STDERR "e.g. x86 or arm"
    exit 1
fi
ANDROID_SDK=${ANDROID_BUILD_FOLDER}/sdk
export ANDROID_SDK_ROOT=${ANDROID_SDK}
export ANDROID_HOME=${ANDROID_SDK}
export PATH=${PATH}:${ANDROID_HOME}/platform-tools
export PATH=${PATH}:${ANDROID_HOME}/tools
export PATH=${PATH}:${ANDROID_HOME}/tools/bin

mkdir -p ${ANDROID_SDK}

TARGET_ARCH=$1

if [ -z "${TARGET_ARCH}" ]; then
    echo STDERR "Missing TARGET_ARCH argument"
    echo STDERR "e.g. x86 or arm"
    exit 1
fi



check_if_emulator_is_running(){
    emus=$(adb devices)
    if [[ ${emus} = *"emulator"* ]]; then
        echo "emulator is running"
        until adb -e shell "ls /storage/emulated/0/"
        do
            echo "waiting emulator FS"
            sleep 30
        done
    else
        echo "emulator is not running"
        exit 1
    fi
}

kill_avd(){
    adb devices | grep emulator | cut -f1 | while read line; do adb -s $line emu kill; done || true
}
delete_existing_avd(){
    kill_avd
    avdmanager delete avd -n ${ABSOLUTE_ARCH}
}

download_emulator() {
    curl -o $HOME/emu.zip https://dl.google.com/android/repository/emulator-linux-5889189.zip
}

create_avd(){

    echo "${GREEN}Creating Android SDK${RESET}"

    yes | sdkmanager --licenses

    if [ ! -d "${ANDROID_SDK}/emulator/" ] ; then
        echo "yes" |
              sdkmanager --no_https \
                "emulator" \
                "platform-tools" \
                "platforms;android-24" \
                "system-images;android-24;default;${ABI}"

        # TODO hack to downgrade Android Emulator. Should be removed as soon as headless mode will be fixed.
        mv $HOME/emu.zip emu.zip
        mv emulator emulator_backup
        unzip emu.zip
        rm emu.zip
    else
        echo "Skipping sdkmanager activity"
    fi

    echo "${BLUE}Creating android emulator${RESET}"

        echo "no" |
             avdmanager -v create avd \
                --name ${ABSOLUTE_ARCH} \
                --package "system-images;android-24;default;${ABI}" \
                -f \
                -c 1000M

        ANDROID_SDK_ROOT=${ANDROID_SDK} ANDROID_HOME=${ANDROID_SDK} ${ANDROID_HOME}/tools/emulator -avd ${ABSOLUTE_ARCH} -no-audio -no-window -no-snapshot -no-accel &
}

download_and_unzip_if_missed() {
    target_dir=$1
    url_pref=$2
    fname=$3
    url="${url_pref}${fname}"
    if [ ! -d "${target_dir}" ] ; then
        echo "${GREEN}Downloading ${fname}${RESET}"
        curl -sSLO ${url}
        unzip -qq ${fname}
        rm ${fname}
        echo "${GREEN}Done!${RESET}"
    else
        echo "${BLUE}Skipping download ${fname}${RESET}"
    fi
}

download_sdk(){
    pushd ${ANDROID_SDK}
        download_and_unzip_if_missed "tools" "https://dl.google.com/android/repository/" "sdk-tools-linux-4333796.zip"
    popd
}

recreate_avd(){
    pushd ${ANDROID_SDK}
        set +e
        delete_existing_avd
        set -e
        create_avd
    popd
}


generate_arch_flags(){
    if [ -z $1 ]; then
        echo STDERR "${RED}Please provide the arch e.g arm,armv7, x86 or arm64${RESET}"
        exit 1
    fi
    export ABSOLUTE_ARCH=$1
    export TARGET_ARCH=$1
    if [ $1 == "arm" ]; then
        export TARGET_API="21"
        export TRIPLET="arm-linux-androideabi"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="armeabi-v7a"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "armv7" ]; then
        export TARGET_ARCH="arm"
        export TARGET_API="21"
        export TRIPLET="armv7-linux-androideabi"
        export ANDROID_TRIPLET="arm-linux-androideabi"
        export ABI="armeabi-v7a"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "arm64" ]; then
        export TARGET_API="21"
        export TRIPLET="aarch64-linux-android"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="arm64-v8a"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "x86" ]; then
        export TARGET_API="21"
        export TRIPLET="i686-linux-android"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="x86"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "x86_64" ]; then
        export TARGET_API="21"
        export TRIPLET="x86_64-linux-android"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="x86_64"
        export TOOLCHAIN_SYSROOT_LIB="lib64"
    fi

}

prepare_dependencies() {
    pushd ${ANDROID_BUILD_FOLDER}
        download_and_unzip_if_missed "openssl_$1" "https://repo.sovrin.org/android/libindy/deps-libc++/openssl/" "openssl_$1.zip"
        download_and_unzip_if_missed "libsodium_$1" "https://repo.sovrin.org/android/libindy/deps-libc++/sodium/" "libsodium_$1.zip"
        download_and_unzip_if_missed "libzmq_$1" "https://repo.sovrin.org/android/libindy/deps-libc++/zmq/" "libzmq_$1.zip"
    popd
}


setup_dependencies_env_vars(){
    export OPENSSL_DIR=${ANDROID_BUILD_FOLDER}/openssl_$1
    export SODIUM_DIR=${ANDROID_BUILD_FOLDER}/libsodium_$1
    export LIBZMQ_DIR=${ANDROID_BUILD_FOLDER}/libzmq_$1
}



create_standalone_toolchain_and_rust_target(){
#    #will only create toolchain if not already created
#    python3 ${ANDROID_NDK_ROOT}/build/tools/make_standalone_toolchain.py \
#    --arch ${TARGET_ARCH} \
#    --api ${TARGET_API} \
#    --stl=libc++ \
#    --force \
#    --install-dir ${TOOLCHAIN_DIR}

    # add rust target
    rustup target add ${TRIPLET}
}



download_and_setup_toolchain(){
    if [ "$(uname)" == "Darwin" ]; then
        export TOOLCHAIN_PREFIX=${ANDROID_BUILD_FOLDER}/toolchains/darwin
        mkdir -p ${TOOLCHAIN_PREFIX}
        pushd $TOOLCHAIN_PREFIX
        echo "${GREEN}Resolving NDK for OSX${RESET}"
        download_and_unzip_if_missed "android-ndk-r20" "https://dl.google.com/android/repository/" "android-ndk-r20-darwin-x86_64.zip"
        popd
    elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
        export TOOLCHAIN_PREFIX=${ANDROID_BUILD_FOLDER}/toolchains/linux
        mkdir -p ${TOOLCHAIN_PREFIX}
        pushd $TOOLCHAIN_PREFIX
        echo "${GREEN}Resolving NDK for Linux${RESET}"
        download_and_unzip_if_missed "android-ndk-r20" "https://dl.google.com/android/repository/" "android-ndk-r20-linux-x86_64.zip"
        popd
    fi
    export ANDROID_NDK_ROOT=${TOOLCHAIN_PREFIX}/android-ndk-r20
}


set_env_vars(){
    if [ -z "${TOOLCHAIN_PREFIX}" ]; then
        echo STDERR "The environment variable \"TOOLCHAIN_PREFIX\" is not set"
        exit 1
    fi
    export PKG_CONFIG_ALLOW_CROSS=1
    export CARGO_INCREMENTAL=1
    export RUST_LOG=indy=trace
    export RUST_TEST_THREADS=1
    export RUST_BACKTRACE=1
    export OPENSSL_DIR=${OPENSSL_DIR}
    export SODIUM_LIB_DIR=${SODIUM_DIR}/lib
    export SODIUM_INCLUDE_DIR=${SODIUM_DIR}/include
    export LIBZMQ_LIB_DIR=${LIBZMQ_DIR}/lib
    export LIBZMQ_INCLUDE_DIR=${LIBZMQ_DIR}/include
    export TOOLCHAIN_DIR=${TOOLCHAIN_PREFIX}/android-ndk-r20/
    export PATH=${TOOLCHAIN_DIR}/toolchains/llvm/prebuilt/linux-x86_64/bin/:${PATH}
    export PKG_CONFIG_ALLOW_CROSS=1
    export CC=${TOOLCHAIN_DIR}/toolchains/llvm/prebuilt/linux-x86_64/bin/${ANDROID_TRIPLET}${TARGET_API}-clang
    export AR=${TOOLCHAIN_DIR}/toolchains/llvm/prebuilt/linux-x86_64/bin/${ANDROID_TRIPLET}-ar
    export CXX=${TOOLCHAIN_DIR}/toolchains/llvm/prebuilt/linux-x86_64/bin/${ANDROID_TRIPLET}${TARGET_API}-clang++
    export CXXLD=${TOOLCHAIN_DIR}/toolchains/llvm/prebuilt/linux-x86_64/bin/${ANDROID_TRIPLET}-ld
    export RANLIB=${TOOLCHAIN_DIR}/toolchains/llvm/prebuilt/linux-x86_64/bin/${ANDROID_TRIPLET}-ranlib
    export TARGET=android
    export OPENSSL_STATIC=1
}
