FROM ubuntu:16.04
LABEL maintainer="Michael Lodder <redmike7@gmail.com>"

ARG target_arch
ARG target_api
ARG cross_compile
ARG sodium_lib_dir

ENV TARGET_ARCH ${target_arch}
ENV TARGET_API ${target_api}
ENV CROSS_COMPILE ${cross_compile}
ENV ZMQ_HAVE_ANDROID 1
ENV SODIUM_LIB_DIR /home/zeromq_user/${sodium_lib_dir}
ENV ANDROID_NDK_ROOT /home/zeromq_user/android-ndk-r20
ENV TOOLCHAIN_DIR /home/zeromq_user/${target_arch}
ENV PATH ${TOOLCHAIN_DIR}/bin:${PATH}

RUN DEBIAN_FRONTEND=noninteractive apt-get -qq update -y && apt-get -qq install -y apt-utils zip unzip autoconf cmake libtool sudo pkg-config wget python3 2>&1 > /dev/null
RUN useradd -m -d /home/zeromq_user -p $(openssl passwd -1 "zeromq") -s /bin/bash zeromq_user
# && mkdir -p /etc/sudoers.d/
RUN usermod -aG sudo zeromq_user
#RUN echo "zeromq_user ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/zeromq_user
#RUN chmod 0440 /etc/sudoers.d/zeromq_user

USER zeromq_user
WORKDIR /home/zeromq_user

COPY android-ndk-r20-linux-x86_64.zip /home/zeromq_user/
COPY zeromq-4.2.5.tar.gz /home/zeromq_user/
COPY ${sodium_lib_dir}/ ${SODIUM_LIB_DIR}/
RUN unzip -qq /home/zeromq_user/android-ndk-r20-linux-x86_64.zip -d /home/zeromq_user/
RUN tar xf /home/zeromq_user/zeromq-4.2.5.tar.gz -C /home/zeromq_user/
RUN python3 ${ANDROID_NDK_ROOT}/build/tools/make_standalone_toolchain.py --arch ${target_arch} --api ${target_api} --install-dir ${TOOLCHAIN_DIR}

WORKDIR /home/zeromq_user/zeromq-4.2.5
RUN ./autogen.sh
RUN ./configure CPP=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-cpp CC=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-clang CXX=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-clang++ LD=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-ld AS=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-as AR=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-ar RANLIB=${TOOLCHAIN_DIR}/bin/${CROSS_COMPILE}-ranlib CFLAGS="-I/home/zeromq_user/libzmq_${TARGET_ARCH}/include -D__ANDROID_API__=${TARGET_API} -fPIC" CXXFLAGS="-I/home/zeromq_user/libzmq_${TARGET_ARCH}/include -D__ANDROID_API__=${TARGET_API} -fPIC" LDFLAGS="-L/home/zeromq_user/libzmq_${TARGET_ARCH}/lib -D__ANDROID_API__=${TARGET_API}" LIBS="-lc -lgcc -ldl" --host=${CROSS_COMPILE} --prefix=/home/zeromq_user/libzmq_${TARGET_ARCH} --with-libsodium=${SODIUM_LIB_DIR} --without-docs --enable-static --with-sysroot=${TOOLCHAIN_DIR}/sysroot
RUN make
RUN make install

WORKDIR /home/zeromq_user
RUN rm -rf libzmq_${TARGET_ARCH}/bin
RUN zip libzmq_${TARGET_ARCH}.zip -r libzmq_${TARGET_ARCH}
RUN echo "libzmq android build successful"
