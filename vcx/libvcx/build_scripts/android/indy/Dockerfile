FROM ubuntu:16.04
LABEL maintainer="Michael Lodder <redmike7@gmail.com>"

ARG target_arch
ARG target_api
ARG cross_compile
ARG openssl_dir
ARG sodium_dir
ARG libzmq_dir
ARG final

ENV TARGET_ARCH ${target_arch}
ENV TARGET_API ${target_api}
ENV CROSS_COMPILE ${cross_compile}
ENV OPENSSL_DIR /home/indy_user/${openssl_dir}
ENV SODIUM_LIB_DIR /home/indy_user/${sodium_dir}/lib
ENV SODIUM_INCLUDE_DIR /home/indy_user/${sodium_dir}/include
ENV LIBZMQ_LIB_DIR /home/indy_user/${libzmq_dir}/lib
ENV LIBZMQ_INCLUDE_DIR /home/indy_user/${libzmq_dir}/include
ENV ANDROID_NDK_ROOT /home/indy_user/android-ndk-r20
ENV TOOLCHAIN_DIR /home/indy_user/${target_arch}
ENV PATH ${TOOLCHAIN_DIR}/bin:${PATH}
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV CC=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-clang
ENV AR=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-ar
ENV CXX=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-clang++
ENV CXXLD=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-ld
ENV RANLIB=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-ranlib
ENV TARGET=android
ENV FINAL=${final}

RUN DEBIAN_FRONTEND=noninteractive apt-get -qq update -y && apt-get -qq install -y zip unzip libtool curl wget python3 pkg-config libzmq3-dev 2>&1 > /dev/null
RUN useradd -m -d /home/indy_user -s /bin/bash indy_user && mkdir -p /etc/sudoers.d/
RUN echo "indy_user ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/indy_user
RUN chmod 0440 /etc/sudoers.d/indy_user

USER indy_user
WORKDIR /home/indy_user

COPY android-ndk-r20-linux-x86_64.zip /home/indy_user/
RUN unzip -qq /home/indy_user/android-ndk-r20-linux-x86_64.zip -d /home/indy_user/
COPY --chown=indy_user:indy_user indy-sdk/ /home/indy_user/indy-sdk/
COPY --chown=indy_user:indy_user ${openssl_dir}/ ${OPENSSL_DIR}/
COPY --chown=indy_user:indy_user ${sodium_dir}/ /home/indy_user/${sodium_dir}/
COPY --chown=indy_user:indy_user ${libzmq_dir}/ /home/indy_user/${libzmq_dir}/
COPY --chown=indy_user:indy_user make_indy.sh /home/indy_user/
RUN chmod a+x make_indy.sh
RUN ./make_indy.sh
RUN echo "libindy android build successful"
