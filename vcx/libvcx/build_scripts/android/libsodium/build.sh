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

if [ ! -f "libsodium-1.0.12.tar.gz" ] ; then
    echo "Downloading libsodium-1.0.12.tar.gz"
    wget -q wget https://github.com/jedisct1/libsodium/releases/download/1.0.12/libsodium-1.0.12.tar.gz
else
    echo "Skipping download libsodium-1.0.12.tar.gz"
fi

sudo docker build -t sodium-android:latest . --build-arg target_arch=${TARGET_ARCH} --build-arg target_api=${TARGET_API} --build-arg cross_compile=${CROSS_COMPILE}

sudo docker run sodium-android:latest && \
docker_id=$(sudo docker ps -a | grep sodium-android:latest | grep Exited | tail -n 1 | cut -d ' ' -f 1) && \
docker_image_id=$(sudo docker image ls | grep sodium-android | perl -pe 's/\s+/ /g' | cut -d ' ' -f 3) && \
sudo docker cp ${docker_id}:/home/sodium_user/libsodium_${TARGET_ARCH}.zip . && \
sudo docker rm ${docker_id} > /dev/null
#sudo docker rmi ${docker_image_id} > /dev/null
