#!/usr/bin/env bash


if [ -z "${ANDROID_BUILD_FOLDER}" ]; then
    echo STDERR "ANDROID_BUILD_FOLDER is not set. Please set it in the caller script"
    echo STDERR "e.g. x86 or arm"
    exit 1
fi
export ANDROID_SDK_ROOT=${ANDROID_BUILD_FOLDER}/sdk
export ANDROID_NDK_ROOT=${ANDROID_SDK_ROOT}/ndk/22.1.7171670
export PATH=${PATH}:${ANDROID_SDK_ROOT}/platform-tools
export PATH=${PATH}:${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin
export PATH=${PATH}:${ANDROID_SDK_ROOT}/emulator

mkdir -p ${ANDROID_SDK_ROOT}

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

create_avd(){

    echo "${GREEN}Creating Android SDK${RESET}"

    yes | sdkmanager --licenses

    echo "yes" |
          sdkmanager --no_https \
            "emulator" \
            "platform-tools" \
            "platforms;android-24" \
            "system-images;android-24;default;${ABI}"

    echo "${BLUE}Creating android emulator${RESET}"

    echo "no" |
         avdmanager -v create avd \
            --name ${ABSOLUTE_ARCH} \
            --package "system-images;android-24;default;${ABI}" \
            -f \
            -c 1000M

    emulator -avd ${ABSOLUTE_ARCH} -no-audio -no-window -no-snapshot -no-accel &
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
    pushd ${ANDROID_SDK_ROOT}
        download_and_unzip_if_missed "cmdline-tools" "https://dl.google.com/android/repository/" "commandlinetools-linux-6858069_latest.zip"

        # Workaround for command line tool issue where it is in incorrect location
        mv cmdline-tools cmdline-tools-tmp
        mkdir -p cmdline-tools/latest
        mv cmdline-tools-tmp/* cmdline-tools/latest
        rm -R cmdline-tools-tmp

        # Accept licanses
        mkdir licenses
        pushd licenses
            # https://developer.android.com/studio/terms
            echo "24333f8a63b6825ea9c5514f83c2829b004d1fee" >> android-sdk-license
        popd
    popd
}

recreate_avd(){
    pushd ${ANDROID_SDK_ROOT}
        set +e
        delete_existing_avd
        set -e
        create_avd
    popd
}

wait_for_emulator(){
    echo "Emulator: Wait for emulator"
    adb wait-for-device

    echo "Emulator: Wait for boot"
    while [ "`adb shell getprop sys.boot_completed | tr -d '\r' `" != "1" ] ;
    do
      echo "Emulator: Waiting for emulator to boot.."
      sleep 5;
    done

    echo "Emulator: Unlock device"
    adb shell input keyevent 82
    echo "Emulator: Device ready"
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
    #will only create toolchain if not already created
    python3 ${ANDROID_NDK_ROOT}/build/tools/make_standalone_toolchain.py \
    --arch ${TARGET_ARCH} \
    --api ${TARGET_API} \
    --stl=libc++ \
    --force \
    --install-dir ${TOOLCHAIN_DIR}

    # add rust target
    rustup target add ${TRIPLET}
}



download_and_setup_toolchain(){
    sdkmanager "ndk;22.1.7171670"
}


set_env_vars(){
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
    export TOOLCHAIN_DIR=${TOOLCHAIN_PREFIX}/${TARGET_ARCH}
    export PATH=${TOOLCHAIN_DIR}/bin:${PATH}
    export PKG_CONFIG_ALLOW_CROSS=1
    export CC=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-clang
    export AR=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-ar
    export CXX=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-clang++
    export CXXLD=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-ld
    export RANLIB=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-ranlib
    export TARGET=android
    export OPENSSL_STATIC=1
}
