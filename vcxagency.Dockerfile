ARG BASE_IMAGE_LIBINDY
FROM ${BASE_IMAGE_LIBINDY}

USER indy
WORKDIR /home/indy/indy-sdk
COPY --chown=indy:indy ./vcx/dummy-cloud-agent ./vcx/dummy-cloud-agent

RUN cargo build --release --manifest-path=/home/indy/indy-sdk/vcx/dummy-cloud-agent/Cargo.toml



