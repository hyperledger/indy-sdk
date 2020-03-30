#!/usr/bin/env bash

set -e
SCRIPT_PATH=${BASH_SOURCE[0]}      # this script's name
SCRIPT_NAME=${SCRIPT_PATH##*/}       # basename of script (strip path)
SCRIPT_DIR="$(cd "$(dirname "${SCRIPT_PATH:-$PWD}")" 2>/dev/null 1>&2 && pwd)"

export ANDROID_BUILD_TOOLS="/home/android/android-sdk-linux/build-tools/27.0.3"
export ANDROID_BUILD_FOLDER="/tmp/android_build"
echo "ANDROID_BUILD_FOLDER: ${ANDROID_BUILD_FOLDER}"
ANDROID_SDK=${ANDROID_BUILD_FOLDER}/sdk
mkdir -p ${ANDROID_SDK}
export ANDROID_SDK_ROOT=${ANDROID_SDK}
export ANDROID_HOME=${ANDROID_SDK}
export PATH=${PATH}:${ANDROID_HOME}/platform-tools
export PATH=${PATH}:${ANDROID_HOME}/tools
export PATH=${PATH}:${ANDROID_HOME}/tools/bin
export ABSOLUTE_ARCH="x86"
export ABI="x86"


create_avd_and_launch_emulator(){

    echo "Creating Android SDK"

    yes | sdkmanager --licenses

    if [ ! -d "${ANDROID_SDK}/emulator/" ] ; then
        echo "yes" |
              sdkmanager --no_https \
                "emulator" \
                "platform-tools" \
                "platforms;android-24" \
                "system-images;android-24;default;${ABI}" > sdkmanager.install.emulator.and.tools.out 2>&1

        # TODO hack to downgrade Android Emulator. Should be removed as soon as headless mode will be fixed.
        curl -o emu.zip https://dl.google.com/android/repository/emulator-linux-5889189.zip
        mv emulator emulator_backup
        unzip emu.zip
    else
        echo "Skipping sdkmanager activity"
    fi

    echo "Creating android emulator"

        echo "no" |
             avdmanager create avd \
                --name ${ABSOLUTE_ARCH} \
                --package "system-images;android-24;default;${ABI}" \
                -f \
                -c 4096M

        ANDROID_SDK_ROOT=${ANDROID_SDK} ANDROID_HOME=${ANDROID_SDK} ${ANDROID_HOME}/tools/emulator -avd ${ABSOLUTE_ARCH} -netdelay none -partition-size 4096 -netspeed full -no-audio -no-window -no-snapshot -no-accel &
}

kill_avd(){
    adb devices | grep emulator | cut -f1 | while read line; do adb -s $line emu kill; done || true
}
delete_existing_avd(){
    kill_avd
    avdmanager delete avd -n ${ABSOLUTE_ARCH}
}

download_and_unzip_if_missed() {
    target_dir=$1
    url_pref=$2
    fname=$3
    url="${url_pref}${fname}"
    if [ ! -d "${target_dir}" ] ; then
        echo "Downloading ${fname}"
        curl -sSLO ${url}
        unzip -qq ${fname}
        rm ${fname}
        echo "Done!"
    else
        echo "Skipping download ${fname}"
    fi
}

download_sdk(){
    pushd ${ANDROID_SDK}
        download_and_unzip_if_missed "tools" "https://dl.google.com/android/repository/" "sdk-tools-linux-4333796.zip"

        set +e
        delete_existing_avd
        set -e
        create_avd_and_launch_emulator
    popd
}

download_sdk

pushd ${SCRIPT_DIR} # we will work on relative paths from the script directory
    pushd ../android
        npm install
    popd

    # This pushd/popd block is for running the tests
    pushd ..
        # ANDROID_JNI_LIB=android/src/main/jniLibs
        # for arch in arm arm64 armv7 x86 x86_64
        # do
        #     arch_folder=${arch}
        #     if [ "${arch}" = "armv7" ]; then
        #         arch_folder="armeabi-v7a"
        #     elif [ "${arch}" = "arm64" ]; then
        #         arch_folder="arm64-v8a"
        #     fi
        #     rm ${ANDROID_JNI_LIB}/${arch_folder}/libc++_shared.so
        # done

        echo "Running :assembleDebugAndroidTest to see if it passes..."

        # Run the tests first
        ./gradlew --full-stacktrace --debug --no-daemon :assembleDebugAndroidTest --project-dir=android -x test

        echo "Installing the android test apk that will test the aar library..."
        i=0
        while
            sleep 10
            : ${start=$i}
            i="$((i+1))"
            ADB_INSTALL=$(adb install ./android/build/outputs/apk/androidTest/debug/com.evernym-vcx_1.0.0-*_x86-armv7-debug-androidTest.apk 2>&1)
            echo "ADB_INSTALL -- ${ADB_INSTALL}"
            FAILED_INSTALL=$(echo ${ADB_INSTALL}|grep "adb: failed to install")
            [ "${FAILED_INSTALL}" != "" ] && [ "$i" -lt 70 ]            # test the limit of the loop.
        do :;  done

        if [ "${FAILED_INSTALL}" != "" ]; then
            exit 1
        fi

        adb shell service list
        echo "Starting the tests of the aar library..."
        ./gradlew --full-stacktrace --debug --no-daemon :connectedCheck --project-dir=android
        cat ./android/build/reports/androidTests/connected/me.connect.VcxWrapperTests.html

        # for arch in arm arm64 armv7 x86 x86_64
        # do
        #     arch_folder=${arch}
        #     if [ "${arch}" = "armv7" ]; then
        #         arch_folder="armeabi-v7a"
        #     elif [ "${arch}" = "arm64" ]; then
        #         arch_folder="arm64-v8a"
        #     fi
        #     cp -v ../../../runtime_android_build/libvcx_${arch}/libc++_shared.so ${ANDROID_JNI_LIB}/${arch_folder}/libc++_shared.so
        # done
    popd
popd

pushd ${SCRIPT_DIR} # we will work on relative paths from the script directory
    pushd ..
        # Now build it clean
        ./gradlew --no-daemon clean build --project-dir=android -x test #skipping tests because they already run in jenkins CI
        mkdir -p artifacts/aar
        pushd android/build/outputs/aar
            cp $(ls -t1 |  head -n 1) ${SCRIPT_DIR}/../artifacts/aar
        popd
    popd
popd
