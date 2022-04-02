FROM debian as base
RUN apt-get update && \
    apt-get install -yqq  curl gcc sudo git && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -d /home/bob -m -G sudo bob
RUN ln -svf /home/bob/bin /,
RUN echo "%sudo  ALL=(ALL) NOPASSWD: ALL" | tee -a /etc/sudoers

RUN curl -LO 'https://github.com/Peltoche/lsd/releases/download/0.21.0/lsd_0.21.0_amd64.deb'
RUN dpkg -i lsd_0.21.0_amd64.deb

USER bob
WORKDIR /home/bob

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM base as build-base
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs --output rustup.sh
RUN bash rustup.sh -y --verbose
RUN $HOME/.cargo/bin/cargo install --bin dev_workspace --git https://gitlab.com/gvz/rs/ --branch main

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM build-base as deps
RUN mkdir -pv release

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM deps as source
ADD _.source.tar release/
RUN sudo chown -Rv bob:bob release

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM source as build
RUN --mount=type=cache,target=$HOME/release/target \
    cd release && \
    $HOME/.cargo/bin/cargo build \
    --features release,serde_cbor \
    --release \
    --bin xs-compile \
    --bin xs-run \
    --bin xs-debug \
    --bin xs-write_out \
    --bin xs-make-self-exec \
    --bin main

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM build as install
RUN cd release && . $HOME/.cargo/env && target/release/main ci/ci-install-system
RUN cd release && xs2 ci/ci-compile

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM debian as dist
RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install -y sudo
RUN echo "%sudo  ALL=(ALL) NOPASSWD: ALL" | tee -a /etc/sudoers
RUN useradd -d /home/bob -m -G sudo bob
USER bob
WORKDIR /home/bob
COPY --from=install /home/bob/release/target/release/xs-compile /usr/bin
COPY --from=install /home/bob/release/target/release/xs-run /usr/bin
COPY --from=install /home/bob/release/target/release/main /usr/bin/xs2
COPY --from=install /home/bob/release/target/release/xs-make-self-exec /usr/bin
COPY --from=install /home/bob/release/target/release/xs-write_out /usr/bin
