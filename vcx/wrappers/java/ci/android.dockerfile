# Development
FROM libindy
ARG uid=1000
RUN useradd -ms /bin/bash -u $uid android
RUN usermod -aG sudo android

RUN apt-get update -y && apt-get install -y \
    openjdk-8-jdk \
    maven

# Install Android SDK and NDK
RUN mkdir -m 777 -p /home/android/android-sdk-linux
RUN wget -q https://dl.google.com/android/repository/tools_r25.2.3-linux.zip -P /home/android/android-sdk-linux
RUN unzip -q /home/android/android-sdk-linux/tools_r25.2.3-linux.zip -d /home/android/android-sdk-linux
RUN yes | .//home/android/android-sdk-linux/tools/android update sdk --no-ui >/dev/null 2>&1
RUN yes | .//home/android/android-sdk-linux/tools/bin/sdkmanager "ndk-bundle" >/dev/null 2>&1

RUN echo "android ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

ARG RUST_VER
USER android
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VER}
ENV PATH /home/android/.cargo/bin:$PATH
