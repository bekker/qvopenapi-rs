# ============== Container setup
FROM scottyhardy/docker-wine:stable-8.0 AS setup

VOLUME /namu/NPKI

USER root

RUN apt-get update

# Basic font files for debugging, window manager for dummy x window
RUN apt-get install -y \
    fonts-nanum fonts-nanum-coding

# Set correct locale
RUN locale-gen ko_KR.UTF-8 && \
    update-locale LANG=ko_KR.UTF-8 LC_MESSAGES=POSIX

# Create wineuser (entrypoint is inherited from docker-wine image)
# Bunch of user creation, permission setup and etc.
RUN /usr/bin/entrypoint echo Done

RUN mkdir -p /namu /home/wineuser/.wine/drive_c/users/wineuser/AppData/LocalLow && chown -R wineuser:wineuser /namu /home/wineuser
WORKDIR /namu

USER wineuser

# ============== Download DLL
FROM setup AS dll

USER root
RUN apt-get install -y curl
USER wineuser

RUN curl -O https://download.nhqv.com/download/iflgtrading/openapi.qv.zip && \
    unzip openapi.qv.zip && \
    cp -R openapi.qv/bin bin && \
    rm -R openapi.qv && rm openapi.qv.zip

# ============== Build
FROM rust:1.69.0-buster AS builder

RUN apt-get update
RUN apt-get install -y gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64

ENV RUST_TARGET="i686-pc-windows-gnu"
RUN rustup target add ${RUST_TARGET}

WORKDIR /usr/src/qvopenapi-rs
COPY . .

RUN cargo update
RUN cargo rustc -p qvopenapi-http --release --target ${RUST_TARGET} --features "disable-unwind" -- -C "panic=abort"

# ============== Container
FROM setup

RUN ln -s /namu/NPKI /home/wineuser/.wine/drive_c/users/wineuser/AppData/LocalLow/NPKI

COPY --from=builder /usr/src/qvopenapi-rs/target/i686-pc-windows-gnu/release /namu/bin
COPY --from=dll /namu/bin /namu/bin
COPY docker/entrypoint.sh /namu/entrypoint.sh

# USER root
# ENTRYPOINT /usr/bin/entrypoint /bin/bash
ENTRYPOINT /namu/entrypoint.sh
