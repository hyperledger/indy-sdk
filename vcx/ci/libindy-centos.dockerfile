# Development
FROM centos:7

ARG LIBINDY_VER

RUN yum install -y https://dl.fedoraproject.org/pub/epel/epel-release-latest-7.noarch.rpm ;\
    yum install -y \
      python3 \
      git \
      zeromq \
      cargo \
      openssl-devel \
      rpm-build \
      https://repo.sovrin.org/rpm/libindy/stable/${LIBINDY_VER}/libindy.${LIBINDY_VER}.rpm
