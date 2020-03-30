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

if [ ! -f "android-ndk-r20-linux-x86_64.zip" ] ; then
    echo "Downloading android-ndk-r20-linux-x86_64.zip"
    wget -q https://dl.google.com/android/repository/android-ndk-r20-linux-x86_64.zip
else
    echo "Skipping download android-ndk-r20-linux-x86_64.zip"
fi

if [ ! -f "openssl-1.1.0h.tar.gz" ] ; then
    echo "Downloading openssl-1.1.0h.tar.gz"
    wget -q https://www.openssl.org/source/openssl-1.1.0h.tar.gz
else
    echo "Skipping download openssl-1.1.0h.tar.gz"
fi

docker build -t openssl-android:latest . --build-arg target_arch=${TARGET_ARCH} --build-arg target_api=${TARGET_API} --build-arg cross_compile=${CROSS_COMPILE} && \
docker run openssl-android:latest && \
docker_id=$(docker ps -a | grep openssl-android:latest | grep Exited | tail -n 1 | cut -d ' ' -f 1) && \
docker_image_id=$(docker image ls | grep openssl-android | perl -pe 's/\s+/ /g' | cut -d ' ' -f 3) && \
docker cp ${docker_id}:/home/openssl_user/openssl_${TARGET_ARCH}.zip . && \
docker rm ${docker_id} > /dev/null && \
docker rmi ${docker_image_id} > /dev/null
