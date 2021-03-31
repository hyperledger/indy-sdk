FROM ubuntu:16.04

RUN apt-get update && \
    apt-get install -y \
      pkg-config \
      libssl-dev \
      libgmp3-dev \
      curl \
      build-essential \
      libsqlite3-dev \
      cmake \
      git \
      python3.5 \
      python3-pip \
      python-setuptools \
      apt-transport-https \
      ca-certificates \
      debhelper \
      wget \
      devscripts \
      libncursesw5-dev \
      libzmq3-dev \
      zip \
      unzip \
      jq


RUN pip3 install -U \
	pip \
	setuptools \
	virtualenv \
	twine \
	plumbum \
	deb-pkg-tools

RUN cd /tmp && \
   curl https://download.libsodium.org/libsodium/releases/libsodium-1.0.18.tar.gz | tar -xz && \
    cd /tmp/libsodium-1.0.18 && \
    ./configure --disable-shared && \
    make && \
    make install && \
    rm -rf /tmp/libsodium-1.0.18

RUN groupadd -g 1000 indy && useradd -r -u 1000 -g indy indy

WORKDIR /home/indy
RUN chown -R indy:indy /home/indy
USER indy

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.43.1
ENV PATH /home/indy/.cargo/bin:$PATH

WORKDIR /home/indy/indy-sdk
COPY --chown=indy:indy ./ ./
#COPY --chown=indy:indy ./libindy ./libindy
#COPY --chown=indy:indy ./wrappers ./wrappers

# TODO :Check that libvcx directory was ignored according to correct dockerignore file
RUN ls -lah /home/indy/indy-sdk
RUN ls -lah /home/indy/indy-sdk/libindy
RUN cargo build --release --manifest-path=/home/indy/indy-sdk/libindy/Cargo.toml
USER root
RUN mv /home/indy/indy-sdk/libindy/target/release/*.so /usr/lib
USER indy
