#!/usr/bin/env bash



#download the ndk
mkdir ${HOME}/.NDK_TOOLCHAINS
export NDK_TOOLCHAIN_DIR=${HOME}/.NDK_TOOLCHAINS
mkdir /tmp/NDK
cd /tmp/NDK

if [ "$(uname)" == "Darwin" ]; then
    echo "Downloading NDK for OSX"
    wget https://dl.google.com/android/repository/android-ndk-r20-darwin-x86_64.zip
    unzip android-ndk-r20-darwin-x86_64.zip
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    echo "Downloading NDK for Linux"
    wget https://dl.google.com/android/repository/android-ndk-r20-linux-x86_64.zip
    unzip android-ndk-r20-linux-x86_64.zip
fi

echo "installing toolchains in directory ${NDK_TOOLCHAIN_DIR}"
android-ndk-r20/build/tools/make_standalone_toolchain.py  --api 21 --arch arm64 --install-dir ${NDK_TOOLCHAIN_DIR}/arm64
android-ndk-r20/build/tools/make_standalone_toolchain.py  --api 14 --arch arm --install-dir ${NDK_TOOLCHAIN_DIR}/arm
android-ndk-r20/build/tools/make_standalone_toolchain.py  --api 14 --arch x86 --install-dir ${NDK_TOOLCHAIN_DIR}/x86

echo "setting up the cargo config file"
cat <<EOF > ~/.cargo/config
[target.aarch64-linux-android]
ar = "${NDK_TOOLCHAIN_DIR}/arm64/bin/aarch64-linux-android-ar"
linker = "${NDK_TOOLCHAIN_DIR}/arm64/bin/aarch64-linux-android-clang"

[target.armv7-linux-androideabi]
ar = "${NDK_TOOLCHAIN_DIR}/arm/bin/arm-linux-androideabi-ar"
linker = "${NDK_TOOLCHAIN_DIR}/arm/bin/arm-linux-androideabi-clang"

[target.i686-linux-android]
ar = "${NDK_TOOLCHAIN_DIR}/x86/bin/i686-linux-android-ar"
linker = "${NDK_TOOLCHAIN_DIR}/x86/bin/i686-linux-android-clang"
EOF


# install target for rust
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android