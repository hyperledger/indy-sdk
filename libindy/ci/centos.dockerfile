FROM centos:7

ARG uid=1000

RUN \
    yum clean all \
    && yum upgrade -y \
    && yum groupinstall -y "Development Tools" \
    && yum install -y epel-release \
    && yum-config-manager --enable epel \
    && yum install -y \
           wget \
           cmake \
           pkgconfig \
           openssl-devel \
           sqlite-devel \
           libsodium-devel \
           spectool \
           rpmrebuild

# install nodejs and npm
RUN curl --silent --location https://rpm.nodesource.com/setup_8.x | bash -
RUN yum -y install nodejs

RUN cd /tmp && \
    curl https://download.libsodium.org/libsodium/releases/libsodium-1.0.18.tar.gz | tar -xz && \
    cd /tmp/libsodium-1.0.18 && \
    ./configure && \
    make && \
    make install && \
    rm -rf /tmp/libsodium-1.0.18

ENV PKG_CONFIG_PATH=$PKG_CONFIG_PATH:/usr/local/lib/pkgconfig
ENV LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/usr/local/lib

RUN yum install -y java-1.8.0-openjdk-devel
ENV JAVA_HOME /usr/lib/jvm/java-1.8.0-openjdk

RUN wget https://repos.fedorapeople.org/repos/dchen/apache-maven/epel-apache-maven.repo -O /etc/yum.repos.d/epel-apache-maven.repo
RUN sed -i s/\$releasever/6/g /etc/yum.repos.d/epel-apache-maven.repo
RUN yum install -y apache-maven

ENV RUST_ARCHIVE=rust-1.39.0-x86_64-unknown-linux-gnu.tar.gz
ENV RUST_DOWNLOAD_URL=https://static.rust-lang.org/dist/$RUST_ARCHIVE

RUN mkdir -p /rust
WORKDIR /rust

RUN curl -fsOSL $RUST_DOWNLOAD_URL \
    && curl -s $RUST_DOWNLOAD_URL.sha256 | sha256sum -c - \
    && tar -C /rust -xzf $RUST_ARCHIVE --strip-components=1 \
    && rm $RUST_ARCHIVE \
    && ./install.sh

ENV PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/root/.cargo/bin"

RUN cd /usr/src && \
    wget https://www.python.org/ftp/python/3.5.2/Python-3.5.2.tgz && \
    tar xzf Python-3.5.2.tgz && \
    cd Python-3.5.2 && \
    ./configure && \
    make altinstall

RUN yum install -y ncurses-devel

RUN cd /tmp && \
    wget https://github.com/zeromq/libzmq/releases/download/v4.2.2/zeromq-4.2.2.tar.gz && \
    tar xfz zeromq-4.2.2.tar.gz && rm zeromq-4.2.2.tar.gz && \
    cd /tmp/zeromq-4.2.2 && \
    ./configure && \
    make && \
    make install && \
    rm -rf /tmp/zeromq-4.2.2

RUN yum install fakeroot -y

RUN useradd -ms /bin/bash -u $uid indy
USER indy

WORKDIR /home/indy
