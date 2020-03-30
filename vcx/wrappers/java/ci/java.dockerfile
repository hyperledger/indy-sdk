# Development
FROM libindy
ARG uid=1000
RUN useradd -ms /bin/bash -u $uid java
RUN usermod -aG sudo java

RUN apt-get update -y && apt-get install -y \
    openjdk-8-jdk \
    maven

RUN echo "java ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers 

USER java