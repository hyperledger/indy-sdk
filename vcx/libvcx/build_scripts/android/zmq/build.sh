#!/bin/bash

TARGET_ARCH=$1
TARGET_API=$2
CROSS_COMPILE=$3

if [ -z "${TARGET_ARCH}" ]; then
    echo STDERR "Missing TARGET_ARCH argument"
    echo STDERR "e.g. x86 or arm"
    exit 1
fi

if [ -z "${TARGET_API}" ]; then
    echo STDERR "Missing TARGET_API argument"
    echo STDERR "e.g. 21"
    exit 1
fi

if [ -z "${CROSS_COMPILE}" ]; then
    echo STDERR "Missing CROSS_COMPILE argument"
    echo STDERR "e.g. i686-linux-android"
    exit 1
fi

if [ -z "${SODIUM_LIB_DIR}" ]; then
    SODIUM_LIB_DIR="libsodium_${TARGET_ARCH}/lib"
    if [ -d "${SODIUM_LIB_DIR}" ] ; then
        echo "Found ${SODIUM_LIB_DIR}"
    elif [ -z "$4" ] ; then
        echo STDERR "Missing SODIUM_LIB_DIR argument and environment variable"
        echo STDERR "e.g. set SODIUM_LIB_DIR=<path> for environment or libsodium_${TARGET_ARCH}/lib"
        exit 1
    else
        SODIUM_LIB_DIR=$4
    fi
fi

if [ ! -f "android-ndk-r20-linux-x86_64.zip" ] ; then
    echo "Downloading android-ndk-r20-linux-x86_64.zip"
    wget -q https://dl.google.com/android/repository/android-ndk-r20-linux-x86_64.zip
else
    echo "Skipping download android-ndk-r20-linux-x86_64.zip"
fi

if [ ! -f "zeromq-4.2.5.tar.gz" ] ; then
    echo "Downloading zeromq-4.2.5.tar.gz"
    wget -q https://github.com/zeromq/libzmq/releases/download/v4.2.5/zeromq-4.2.5.tar.gz
else
    echo "Skipping download zeromq-4.2.5.tar.gz"
fi


docker build -t zeromq-android:latest . --build-arg target_arch=${TARGET_ARCH} --build-arg target_api=${TARGET_API} --build-arg cross_compile=${CROSS_COMPILE} --build-arg sodium_lib_dir=${SODIUM_LIB_DIR} && \
docker run zeromq-android:latest && \
docker_id=$(docker ps -a | grep zeromq-android:latest | grep Exited | tail -n 1 | cut -d ' ' -f 1) && \
docker_image_id=$(docker image ls | grep zeromq-android | perl -pe 's/\s+/ /g' | cut -d ' ' -f 3) && \
docker cp ${docker_id}:/home/zeromq_user/libzmq_${TARGET_ARCH}.zip . && \
docker rm ${docker_id} > /dev/null && \
docker rmi ${docker_image_id} > /dev/null
