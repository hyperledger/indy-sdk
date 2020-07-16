FROM ubuntu:18.04

ARG uid=1000

RUN apt-get update && \
    apt-get install -y \
      pkg-config \
      libssl-dev \
      curl \
      libsqlite3-dev \
      cmake \
      python3-pip \
      debhelper \
      devscripts \
      libncursesw5-dev \
      libzmq3-dev \
      libsodium-dev

RUN pip3 install -U \
	pip \
	twine \
	plumbum==1.6.7 six==1.12.0 \
	deb-pkg-tools

RUN apt-get update && apt-get install -y --no-install-recommends \
        ruby \
        ruby-dev \
        rubygems \
    && gem install --no-ri --no-rdoc rake fpm \
    && rm -rf /var/lib/apt/lists/*

# install java and maven
RUN apt-get update && apt-get install openjdk-8-jdk -y
ENV JAVA_HOME /usr/lib/jvm/java-8-openjdk-amd64
RUN apt-get update && apt-get install -y maven

# install nodejs and npm
RUN curl -sL https://deb.nodesource.com/setup_10.x | bash -
RUN apt-get install -y nodejs

RUN apt-get install -y wget

RUN useradd -ms /bin/bash -u $uid indy
USER indy

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.44.1
ENV PATH /home/indy/.cargo/bin:$PATH

RUN cargo install cargo-deb

WORKDIR /home/indy
