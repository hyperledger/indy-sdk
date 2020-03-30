FROM ubuntu:16.04
LABEL maintainer="Michael Lodder <redmike7@gmail.com>"

ARG target_arch
ARG target_api
ARG cross_compile
ARG openssl_dir
ARG sodium_dir
ARG libzmq_dir
ARG libindy_dir

ENV TARGET_ARCH ${target_arch}
ENV TARGET_API ${target_api}
ENV CROSS_COMPILE ${cross_compile}
ENV OPENSSL_DIR /home/vcx_user/${openssl_dir}
ENV SODIUM_LIB_DIR /home/vcx_user/${sodium_dir}/lib
ENV SODIUM_INCLUDE_DIR /home/vcx_user/${sodium_dir}/include
ENV LIBZMQ_LIB_DIR /home/vcx_user/${libzmq_dir}/lib
ENV LIBZMQ_INCLUDE_DIR /home/vcx_user/${libzmq_dir}/include
ENV LIBINDY_DIR /home/vcx_user/${libindy_dir}
ENV ANDROID_NDK_ROOT /home/vcx_user/android-ndk-r20
ENV TOOLCHAIN_DIR /home/vcx_user/${target_arch}
ENV PATH ${TOOLCHAIN_DIR}/bin:${PATH}
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV CC=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-clang
ENV AR=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-ar
ENV CXX=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-clang++
ENV CXXLD=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-ld
ENV RANLIB=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-ranlib
ENV TARGET=android

RUN DEBIAN_FRONTEND=noninteractive apt-get -qq update -y && apt-get -qq install -y zip unzip libtool curl wget python3 2>&1 > /dev/null
RUN useradd -m -d /home/vcx_user -s /bin/bash vcx_user && mkdir -p /etc/sudoers.d/
RUN echo "vcx_user ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/vcx_user
RUN chmod 0440 /etc/sudoers.d/vcx_user

USER vcx_user
WORKDIR /home/vcx_user

COPY android-ndk-r20-linux-x86_64.zip /home/vcx_user/
RUN unzip -qq /home/vcx_user/android-ndk-r20-linux-x86_64.zip -d /home/vcx_user/
COPY --chown=vcx_user:vcx_user sdk/ /home/vcx_user/sdk/
COPY --chown=vcx_user:vcx_user ${openssl_dir}/ ${OPENSSL_DIR}/
COPY --chown=vcx_user:vcx_user ${sodium_dir}/ /home/vcx_user/${sodium_dir}/
COPY --chown=vcx_user:vcx_user ${libzmq_dir}/ /home/vcx_user/${libzmq_dir}/
COPY --chown=vcx_user:vcx_user ${libindy_dir}/ /home/vcx_user/${libindy_dir}/
COPY --chown=vcx_user:vcx_user make_vcx.sh /home/vcx_user/
RUN chmod a+x make_vcx.sh
RUN ./make_vcx.sh
RUN echo "libvcx android build successful"
