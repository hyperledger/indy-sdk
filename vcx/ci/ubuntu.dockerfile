# Development
FROM ubuntu:16.04

# JRE installation and gcc
RUN apt-get update -y && apt-get install -y \
    gcc \
    pkg-config \
    build-essential \
    libsodium-dev \
    libssl-dev \
    libgmp3-dev \
    build-essential \
    libsqlite3-dev \
    libsqlite0 \
    cmake \
    apt-transport-https \
    ca-certificates \
    software-properties-common \
    debhelper \
    wget \
    git \
    curl \
	libffi-dev \
    ruby \
    ruby-dev \
	sudo \
    rubygems \
    libzmq5 \
    python3

# Install Nodejs
RUN curl -sL https://deb.nodesource.com/setup_8.x | bash - \
    && apt-get install -y nodejs

# fpm for deb packaging of npm
RUN gem install fpm
RUN apt-get install rpm -y

COPY ./vcx/ci/scripts/installCert.sh /tmp
RUN /tmp/installCert.sh

# Add sovrin to sources.list
RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys CE7709D068DB5E88 && \
    add-apt-repository "deb https://repo.sovrin.org/sdk/deb xenial master" && \
    add-apt-repository "deb https://repo.sovrin.org/sdk/deb xenial stable" && \
    add-apt-repository 'deb https://repo.sovrin.org/deb xenial master'

ARG LIBINDY_VER="1.13.0"
ARG LIBNULL_VER="1.13.0"

RUN apt-get update && apt-get install -y \
    libindy=${LIBINDY_VER} \
    libnullpay=${LIBNULL_VER}


RUN mkdir -p /build
WORKDIR /build

ARG uid=1000
RUN useradd -ms /bin/bash -u $uid vcx
USER vcx

ARG RUST_VER="1.44.1"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $RUST_VER
ENV PATH /home/vcx/.cargo/bin:$PATH

# Install clippy to the Rust toolchain
RUN rustup component add clippy

RUN cargo install cargo-deb --color=never

CMD ["sh", "vcx/ci/scripts/package.sh"]


