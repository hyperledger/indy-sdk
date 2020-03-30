FROM ubuntu:16.04
LABEL maintainer="Michael Lodder <redmike7@gmail.com>"

ARG target_arch
ARG target_api
ARG cross_compile

ENV TARGET_ARCH ${target_arch}
ENV TARGET_API ${target_api}
ENV CROSS_COMPILE ${cross_compile}
ENV ANDROID_NDK_ROOT /home/openssl_user/android-ndk-r20
ENV TOOLCHAIN_DIR /home/openssl_user/${target_arch}

RUN DEBIAN_FRONTEND=noninteractive apt-get -qq update -y && apt-get -qq install -y zip unzip autoconf cmake wget python3 2>&1 > /dev/null
RUN useradd -m -d /home/openssl_user -s /bin/bash openssl_user && mkdir -p /etc/sudoers.d/
RUN echo "openssl_user ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/openssl_user
RUN chmod 0440 /etc/sudoers.d/openssl_user

USER openssl_user
WORKDIR /home/openssl_user
COPY make_openssl.sh /home/openssl_user/
COPY android-ndk-r20-linux-x86_64.zip /home/openssl_user/
COPY openssl-1.1.0h.tar.gz /home/openssl_user/
RUN unzip -qq /home/openssl_user/android-ndk-r20-linux-x86_64.zip -d /home/openssl_user/
RUN tar xf /home/openssl_user/openssl-1.1.0h.tar.gz -C /home/openssl_user/
RUN python3 ${ANDROID_NDK_ROOT}/build/tools/make_standalone_toolchain.py --arch ${TARGET_ARCH} --api ${TARGET_API} --install-dir ${TOOLCHAIN_DIR}
RUN bash make_openssl.sh
