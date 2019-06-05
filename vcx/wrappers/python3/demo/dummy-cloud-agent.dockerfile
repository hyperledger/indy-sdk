FROM ubuntu:16.04

# Standards packages

RUN apt-get update -y && apt-get install -y \
        gcc \
        wget \
        vim \
        curl \
        apt-transport-https \
        software-properties-common \
        libssl1.0.0 \
        libsqlite0 \
        libzmq5 \
        build-essential

RUN useradd -ms /bin/bash vcx

# Switch to user "vcx"

USER vcx
WORKDIR /home/vcx

# Install Rust

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.31.0
ENV PATH /home/vcx/.cargo/bin:$PATH

# Copy libindy (see README.md for informations)

COPY libindy.so /home/vcx/libindy/libindy.so
ENV LIBRARY_PATH=/home/vcx/libindy
ENV LD_LIBRARY_PATH=/home/vcx/libindy

# Install Cargo deb

RUN cargo install cargo-deb --color=never
