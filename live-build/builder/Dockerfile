# rust:1.90.0-bookworm, linux/amd64 manifest digest. Keep the builder input
# immutable; update this digest only with a deliberate toolchain review.
FROM --platform=linux/amd64 rust@sha256:3f6e6f8d8725a65a2db964bb828850f888d430c68784d661f753144e5d787207

ARG DEBIAN_FRONTEND=noninteractive
ENV DEBIAN_SNAPSHOT=20260801T000000Z

RUN set -eux; \
    for source in /etc/apt/sources.list /etc/apt/sources.list.d/*.sources; do \
      [ -f "$source" ] || continue; \
      sed -i \
        -e "s|https\?://deb.debian.org/debian-security|https://snapshot.debian.org/archive/debian-security/${DEBIAN_SNAPSHOT}|g" \
        -e "s|https\?://security.debian.org/debian-security|https://snapshot.debian.org/archive/debian-security/${DEBIAN_SNAPSHOT}|g" \
        -e "s|https\?://deb.debian.org/debian|https://snapshot.debian.org/archive/debian/${DEBIAN_SNAPSHOT}|g" \
        "$source"; \
    done; \
    apt-get -o Acquire::Check-Valid-Until=false update \
    && apt-get install -y --no-install-recommends \
      ca-certificates \
      coreutils \
      cryptsetup \
      debootstrap \
      e2fsprogs \
      fdisk \
      git \
      live-build \
      qemu-system-x86 \
      squashfs-tools \
      util-linux \
      xorriso \
    && rustup target add x86_64-unknown-linux-musl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
CMD ["sh"]
