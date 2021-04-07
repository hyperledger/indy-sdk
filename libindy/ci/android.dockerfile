FROM libindy-test
# to see base image for this one see this file: libindy/ci/ubuntu.dockerfile. It is build in CI/CD pipelines
ENV ANDROID_BUILD_FOLDER=/tmp/android_build
ENV ANDROID_SDK_ROOT=${ANDROID_BUILD_FOLDER}/sdk
ENV ANDROID_NDK_HOME=${ANDROID_SDK_ROOT}/ndk/22.1.7171670
ENV TOOLCHAIN_PREFIX=${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64
ENV PATH=${PATH}:${ANDROID_SDK_ROOT}/platform-tools:${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin

COPY android.prepare.sh .
COPY setup.android.env.sh .
USER root
RUN chmod +x android.prepare.sh
RUN chown indy:indy android.prepare.sh
USER indy
RUN ["./android.prepare.sh", "subarch"]
