FROM libindy
ARG uid=1000
RUN useradd -ms /bin/bash -u $uid python
RUN echo "python ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

RUN apt-get update && apt-get install -y python3

RUN apt-get install -y python3-pip

RUN pip3 install pytest==5.2.0 qrcode pytest-asyncio requests

ENV PYTHONPATH=vcx/wrappers/python3

RUN find . -name \*.pyc -delete
USER python
