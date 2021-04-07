#!/bin/bash
set -e
export TARGET_ARCH=$2

export ANDROID_BUILD_FOLDER="/tmp/android_build"
source setup.android.env.sh

echo ">> in runner script"

archs=("arm" "armv7" "x86" "arm64" "x86_64")

download_sdk
download_and_setup_toolchain

for arch in "${archs[@]}"
do
  prepare_dependencies "${arch}"
done