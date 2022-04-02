# vim: et ts=4 sw=4

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$


# ###
# Debian
# ###

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
RUN cd release && \
    $HOME/.cargo/bin/cargo build \
    --features release,serde_cbor \
    --release \
    --bin xs-compile \
    --bin xs-run \
    --bin xs-debug \
    --bin xs-write_out \
    --bin xs-make-self-exec \
    --bin xs2

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM build as install
RUN cd release && . $HOME/.cargo/env && target/release/xs2 ci/ci install::system
RUN cd release && xs2 ci/ci CONFIG_DIR/ bob

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM debian as debian-dist
RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install -y sudo
RUN echo "%sudo  ALL=(ALL) NOPASSWD: ALL" | tee -a /etc/sudoers
RUN useradd -d /home/bob -m -G sudo bob
USER bob
WORKDIR /home/bob
COPY --from=install /home/bob/release/target/release/xs-compile /usr/bin
COPY --from=install /home/bob/release/target/release/xs-run /usr/bin
COPY --from=install /home/bob/release/target/release/xs2 /usr/bin/
COPY --from=install /home/bob/release/target/release/xs-make-self-exec /usr/bin
COPY --from=install /home/bob/release/target/release/xs-write_out /usr/bin
COPY --from=install /home/bob/release/target/release/xs-write_out /usr/bin/xs-write_in




###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$


# ###
# Archlinux
# ###


FROM archlinux as arch-base
RUN pacman -Syu --noconfirm curl gcc sudo git lsd

RUN useradd -d /home/bob -m bob
RUN ln -svf /home/bob/bin /,
RUN echo "bob ALL=(ALL) NOPASSWD: ALL" | tee -a /etc/sudoers
RUN sudo -u bob sudo ls -l

USER bob
WORKDIR /home/bob

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM arch-base as arch-build-base
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs --output rustup.sh
RUN bash rustup.sh -y --verbose
RUN $HOME/.cargo/bin/cargo install --bin dev_workspace --git https://gitlab.com/gvz/rs/ --branch main

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM arch-build-base as arch-deps
RUN mkdir -pv release

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM arch-deps as arch-source
ADD _.source.tar release/
RUN sudo chown -Rv bob:bob release

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM arch-source as arch-build
RUN cd release && \
    $HOME/.cargo/bin/cargo build \
    --features release,serde_cbor \
    --release \
    --bin xs-compile \
    --bin xs-run \
    --bin xs-debug \
    --bin xs-write_out \
    --bin xs-make-self-exec \
    --bin xs2
RUN cp -av \
        release/target/release/xs-run \
        release/target/release/xs-compile \
        release/target/release/xs-debug \
        release/target/release/xs-write_out \
        release/target/release/xs-make-self-exec \
        release/target/release/xs2 \
        $HOME/.cargo/bin/ \
    && cp -av \
        release/target/release/xs-write_out $HOME/.cargo/bin/xs-write_in

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM arch-build as arch-install
RUN cd release && . $HOME/.cargo/env && xs2 ci/ci install::system
RUN cd release && xs2 ci/ci CONFIG_DIR/ bob

###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$

FROM archlinux as arch-dist
RUN pacman -Syu --noconfirm sudo
RUN useradd -d /home/bob -m bob
RUN ln -svf /home/bob/bin /,
RUN echo "bob ALL=(ALL) NOPASSWD: ALL" | tee -a /etc/sudoers
USER bob
WORKDIR /home/bob
COPY --from=arch-install /home/bob/release/target/release/xs-compile /usr/bin
COPY --from=arch-install /home/bob/release/target/release/xs-run /usr/bin
COPY --from=arch-install /home/bob/release/target/release/xs2 /usr/bin/xs2
COPY --from=arch-install /home/bob/release/target/release/xs-make-self-exec /usr/bin
COPY --from=arch-install /home/bob/release/target/release/xs-write_out /usr/bin
COPY --from=arch-install /home/bob/release/target/release/xs-write_out /usr/bin/xs-write_in



###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$


# ###
# Meta targets
# ###
FROM scratch as all-dist
COPY --from=arch-dist /usr/bin/xs-compile           /var/xs-arch/
COPY --from=arch-dist /usr/bin/xs-run               /var/xs-arch/
COPY --from=arch-dist /usr/bin/xs2                  /var/xs-arch/
COPY --from=arch-dist /usr/bin/xs-make-self-exec    /var/xs-arch/
COPY --from=arch-dist /usr/bin/xs-write_out         /var/xs-arch/
COPY --from=arch-dist /usr/bin/xs-write_out         /var/xs-arch/
COPY --from=debian-dist /usr/bin/xs-compile         /var/xs-debian/
COPY --from=debian-dist /usr/bin/xs-run             /var/xs-debian/
COPY --from=debian-dist /usr/bin/xs2                /var/xs-debian/
COPY --from=debian-dist /usr/bin/xs-make-self-exec  /var/xs-debian/
COPY --from=debian-dist /usr/bin/xs-write_out       /var/xs-debian/
COPY --from=debian-dist /usr/bin/xs-write_out       /var/xs-debian/
