FROM rust:1.90

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
      ca-certificates \
      coreutils \
      cryptsetup \
      debootstrap \
      e2fsprogs \
      fdisk \
      live-build \
      qemu-system-x86 \
      squashfs-tools \
      util-linux \
      xorriso \
    && rustup target add x86_64-unknown-linux-musl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
CMD ["sh"]
