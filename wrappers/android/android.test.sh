#!/usr/bin/env bash

WORKDIR=${PWD}
LIBINDY_WORKDIR="${WORKDIR}/../../libindy"
CI_DIR="${LIBINDY_WORKDIR}/ci"
BUILD_TYPE="--release"

TARGET_ARCH=x86

set -e

source ${CI_DIR}/setup.android.env.sh $TARGET_ARCH
generate_arch_flags $TARGET_ARCH

echo ">> in runner script"
WORKDIR=${PWD}

# JAVA_OPTS="-Xmx4000M -XX:MaxPermSize=512m -XX:+HeapDumpOnOutOfMemoryError -Dfile.encoding=UTF-8"

./gradlew clean --no-daemon
./gradlew assembleAndroidTest -S --no-daemon

recreate_avd
wait_for_emulator
adb install build/outputs/apk/androidTest/debug/libindy-android-debug-androidTest.apk
adb shell am instrument -e TEST_POOL_IP 10.0.0.2 -w org.hyperledger.android.indy.test/androidx.test.runner.AndroidJUnitRunner
#./gradlew -Pandroid.testInstrumentationRunnerArguments.TEST_POOL_IP=10.0.0.2 connectedCheck -S --no-daemon

kill_avd
