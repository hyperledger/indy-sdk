FROM ubuntu:16.04

# Standards packages

RUN apt-get update
RUN apt-get install -y \
        wget \
        sudo \
        apt-transport-https \
        software-properties-common \
        python3 \
        python3-pip

RUN pip3 install pytest==3.4.2 qrcode pytest-asyncio

# Add repo for libvcx and install libvcx

RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 68DB5E88
RUN add-apt-repository "deb https://repo.sovrin.org/sdk/deb xenial master"
RUN apt-get update
RUN apt-get install -y libvcx

RUN pip3 install python3-wrapper-vcx

# Create user 'vcx'

RUN useradd -ms /bin/bash vcx
USER vcx
WORKDIR /home/vcx
