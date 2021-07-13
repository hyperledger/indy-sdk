FROM ubuntu:16.04

ARG uid=1000
RUN useradd -ms /bin/bash -u $uid indy

RUN apt-get update && apt-get upgrade -y

RUN apt-get update && apt-get install -y \
        curl \
        git

# Kcov build deps
RUN apt-get update && apt-get install -y \
        binutils-dev \
        build-essential \
        cmake \
        libcurl4-openssl-dev \
        libdw-dev \
        libiberty-dev \
        ninja-build \
        python \
        zlib1g-dev

# Kcov installation
RUN git clone 'https://github.com/SimonKagstrom/kcov.git' && \
    cd kcov && \
    mkdir -p build && \
    cd build && \
    cmake .. && \
    make && \
    make install

# Indy Dependecies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    libzmq3-dev \
    libncursesw5-dev

RUN cd /tmp && \
   curl https://download.libsodium.org/libsodium/releases/libsodium-1.0.17.tar.gz | tar -xz && \
    cd /tmp/libsodium-1.0.17 && \
    ./configure --disable-shared && \
    make && \
    make install && \
    rm -rf /tmp/libsodium-1.0.17

# Run as user `indy`
USER indy
WORKDIR /home/indy

# Rust installation
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.31.0
ENV PATH /home/indy/.cargo/bin:$PATH
ENV CARGO_HOME /home/indy/.cargo
