# vim: et ts=4 sw=4
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$


###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
#
# Debian
#
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM debian as debian-base
RUN apt-get update && \
    apt-get install -yqq  curl gcc sudo git && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -d /home/bob -m -G sudo bob
RUN echo "%sudo  ALL=(ALL) NOPASSWD: ALL" | tee -a /etc/sudoers

RUN curl -LO 'https://github.com/Peltoche/lsd/releases/download/0.21.0/lsd_0.21.0_amd64.deb'
RUN dpkg -i lsd_0.21.0_amd64.deb

USER bob
WORKDIR /home/bob
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM debian-base as debian-build-base
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs --output rustup.sh
RUN bash rustup.sh -y --verbose
RUN $HOME/.cargo/bin/cargo install \
    --bin dev_workspace \
    --git https://gitlab.com/gvz/rs/ --branch main
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM debian-build-base as debian-deps
RUN mkdir -pv release
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM debian-deps as debian-source
ADD _.source.tar release/
RUN sudo chown -Rv bob:bob release
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM debian-source as debian-build
RUN cd release && . $HOME/.cargo/env && cargo build --release
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM debian-build as debian-install
ADD ci release/ci
RUN cd release && . $HOME/.cargo/env && ci/bootstrap.sh
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM debian as debian-dist
RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install -y sudo
RUN echo "%sudo  ALL=(ALL) NOPASSWD: ALL" | tee -a /etc/sudoers
RUN useradd -d /home/bob -m -G sudo bob
COPY --from=debian-install /home/bob/release/bin-release.tar /tmp/dust-bin-release.tar
RUN tar -C /usr -xf /tmp/dust-bin-release.tar && rm /tmp/dust-bin-release.tar
USER bob
WORKDIR /home/bob
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$


###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
#
# Archlinux
#
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM archlinux as arch-base
RUN pacman -Syu --noconfirm curl gcc sudo git lsd

RUN useradd -d /home/bob -m bob
RUN echo "bob ALL=(ALL) NOPASSWD: ALL" | tee -a /etc/sudoers
RUN sudo -u bob sudo ls -l

USER bob
WORKDIR /home/bob
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM arch-base as arch-build-base
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs --output rustup.sh
RUN bash rustup.sh -y --verbose \
    --default-toolchain nightly
RUN $HOME/.cargo/bin/cargo install \
    --bin dev_workspace \
    --git https://gitlab.com/gvz/rs/ --branch main
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM arch-build-base as arch-deps
RUN mkdir -pv release
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM arch-deps as arch-source
ADD _.source.tar release/
RUN sudo chown -Rv bob:bob release
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM arch-source as arch-build
RUN cd release && . $HOME/.cargo/env && cargo build --release
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM arch-build as arch-install
ADD ci release/ci
RUN cd release && . $HOME/.cargo/env && ci/bootstrap.sh
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM archlinux as arch-dist
RUN pacman -Syu --noconfirm sudo
RUN useradd -d /home/bob -m bob
RUN ln -svf /home/bob/bin /,
RUN echo "bob ALL=(ALL) NOPASSWD: ALL" | tee -a /etc/sudoers
COPY --from=arch-install /home/bob/release/bin-release.tar /tmp/dust-bin-release.tar
RUN tar -C /usr -xf /tmp/dust-bin-release.tar && rm /tmp/dust-bin-release.tar
USER bob
WORKDIR /home/bob
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$


###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
#
# Alpine Linux
#
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM alpine as alp-base
RUN apk add curl gcc git sudo
RUN addgroup -S sudo
RUN adduser -h /home/bob -G sudo -D -u 1000 bob
RUN echo "%sudo  ALL=(ALL) NOPASSWD: ALL" | tee -a /etc/sudoers
USER bob
WORKDIR /home/bob
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM alp-base as alp-build-base
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs --output rustup.sh
RUN sh rustup.sh -y --verbose
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM alp-build-base as alp-deps
RUN mkdir -pv release
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM alp-deps as alp-source
ADD _.source.tar release/
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM alp-source as alp-build
RUN cd release && . $HOME/.cargo/env && cargo build --release
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM alp-build as alp-install
ADD ci release/ci
RUN cd release && . $HOME/.cargo/env && ci/bootstrap.sh
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM alpine as alp-dist
COPY --from=alp-install /release/bin-release.tar /tmp/dust-bin-release.tar
RUN tar -C /usr -xf /tmp/dust-bin-release.tar && rm /tmp/dust-bin-release.tar
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$


###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
#
# Meta targets
#
###$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
FROM scratch as all-dist
COPY --from=arch-dist   /usr/bin/xs-call /dev/null
COPY --from=debian-dist /usr/bin/xs-call /dev/null
COPY --from=alp-dist    /usr/bin/xs-call /dev/null
