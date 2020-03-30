FROM ubuntu:16.04
LABEL maintainer="Michael Lodder <redmike7@gmail.com>"

ARG target_arch
ARG target_api
ARG cross_compile

ENV TARGET_ARCH ${target_arch}
ENV TARGET_API ${target_api}
ENV CROSS_COMPILE ${cross_compile}
ENV ANDROID_NDK_ROOT /home/sodium_user/android-ndk-r20
ENV TOOLCHAIN_DIR /home/sodium_user/${target_arch}
ENV PATH ${TOOLCHAIN_DIR}/bin:${PATH}
ENV CC ${TOOLCHAIN_DIR}/bin/${cross_compile}-clang
ENV AR ${TOOLCHAIN_DIR}/bin/${cross_compile}-ar
ENV CXX ${TOOLCHAIN_DIR}/bin/${cross_compile}-clang++
ENV CXXLD ${TOOLCHAIN_DIR}/bin/${cross_compile}-ld
ENV RANLIB ${TOOLCHAIN_DIR}/bin/${cross_compile}-ranlib

RUN DEBIAN_FRONTEND=noninteractive apt-get -qq update -y && apt-get -qq install -y apt-utils zip unzip autoconf cmake libtool wget sudo pkg-config python3 2>&1 > /dev/null
RUN useradd -m -d /home/sodium_user -p $(openssl passwd -1 "sodium") -s /bin/bash sodium_user
# && mkdir -p /etc/sudoers.d/
RUN usermod -aG sudo sodium_user
#RUN echo "sodium_user ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/sodium_user
#RUN chmod 0440 /etc/sudoers.d/sodium_user

USER sodium_user
WORKDIR /home/sodium_user

COPY android-ndk-r20-linux-x86_64.zip /home/sodium_user/
COPY libsodium-1.0.12.tar.gz /home/sodium_user/
RUN unzip -qq /home/sodium_user/android-ndk-r20-linux-x86_64.zip -d /home/sodium_user/
RUN tar xf /home/sodium_user/libsodium-1.0.12.tar.gz -C /home/sodium_user/
RUN python3 ${ANDROID_NDK_ROOT}/build/tools/make_standalone_toolchain.py --arch ${target_arch} --api ${target_api} --install-dir ${TOOLCHAIN_DIR}

WORKDIR /home/sodium_user/libsodium-1.0.12
RUN ./autogen.sh
RUN ./configure --prefix=/home/sodium_user/libsodium_${TARGET_ARCH} --disable-soname-versions --host=${CROSS_COMPILE}
RUN make
RUN make install

WORKDIR /home/sodium_user
RUN zip libsodium_${TARGET_ARCH}.zip -r libsodium_${TARGET_ARCH}
RUN echo "libsodium android build successful"
