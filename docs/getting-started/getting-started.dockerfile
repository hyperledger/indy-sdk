FROM ubuntu:16.04

RUN useradd -ms /bin/bash indy

# Install environment
RUN apt-get update -y && apt-get install -y \
	wget \
	python3.5 \
	python3-pip \
	python3-dev \
	python-setuptools \
	apt-transport-https \
	ca-certificates \
	software-properties-common

WORKDIR /home/indy

RUN python3 --version
RUN pip3 install --upgrade pip==9.0.3
RUN pip3 install -U setuptools
RUN pip3 install --upgrade pip
RUN pip3 install -U ipython==7.9
RUN pip3 install -U notebook
RUN pip3 install -U jupyter
RUN pip3 install -U python3-indy==1.11.0


RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys CE7709D068DB5E88 \
    && add-apt-repository "deb https://repo.sovrin.org/sdk/deb xenial stable" \
    && apt-get update \
    && apt-get install -y \
    libindy=1.11.0

USER indy

EXPOSE 8888
