FROM ubuntu:16.04

RUN useradd -ms /bin/bash indy

# Install environment
RUN apt-get update -y && apt-get install -y \
	wget \
	python3.5 \
	python3-pip \
	python-setuptools \
	apt-transport-https \
	ca-certificates \
	software-properties-common

WORKDIR /home/indy

RUN pip3 install --upgrade pip
RUN pip3 install -U \
	pip \
	notebook \
      	ipython==7.9 \
	setuptools \
	jupyter \
	python3-indy==1.11.0

RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys CE7709D068DB5E88 \
    && add-apt-repository "deb https://repo.sovrin.org/sdk/deb xenial stable" \
    && apt-get update \
    && apt-get install -y \
    libindy=1.11.0

USER indy

EXPOSE 8888
