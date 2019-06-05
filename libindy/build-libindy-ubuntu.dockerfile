FROM ubuntu:16.04 as build-libindy

RUN apt-get update -y && apt-get install -y \
        curl \
        build-essential \
        pkg-config \
        cmake \
        libssl-dev \
        libsqlite3-dev \
        libzmq3-dev \
        libncursesw5-dev

# Install libsodium

RUN cd /tmp && \
   curl https://download.libsodium.org/libsodium/releases/old/libsodium-1.0.14.tar.gz | tar -xz && \
    cd /tmp/libsodium-1.0.14 && \
    ./configure --disable-shared && \
    make && \
    make install && \
    rm -rf /tmp/libsodium-1.0.14

# Switch to user "libindy"

RUN useradd -ms /bin/bash libindy
USER libindy
WORKDIR /home/libindy

# Install Rust

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.31.0
ENV PATH /home/libindy/.cargo/bin:$PATH

# Copy sources

COPY libindy/Cargo.toml Cargo.toml
COPY libindy/Cargo.lock Cargo.lock
COPY libindy/build.rs build.rs
COPY libindy/src src
COPY libindy/benches benches
COPY wrappers ../wrappers

# Build libindy
RUN cargo build --features sodium_static

