# syntax=docker/dockerfile:1

FROM --platform=$BUILDPLATFORM tonistiigi/xx:1.9.0@sha256:c64defb9ed5a91eacb37f96ccc3d4cd72521c4bd18d5442905b95e2226b0e707 AS xx

FROM --platform=$BUILDPLATFORM rust:1.97.1-bookworm@sha256:77fac8b98f9f46062bb680b6d25d5bcaabfc400143952ebc572e924bcbedc3fa AS builder

ARG DIOXUS_CLI_VERSION=0.7.9
RUN cargo install dioxus-cli --version $DIOXUS_CLI_VERSION --locked

COPY --from=xx / /

RUN apt-get update && \
    apt-get install -y --no-install-recommends clang lld && \
    rm -rf /var/lib/apt/lists/*

ARG TARGETPLATFORM

RUN xx-apt-get update && \
    xx-apt-get install -y --no-install-recommends gcc libc6-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY . .

RUN xx-cargo --setup-target-triple && \
    target="$(xx-cargo --print-target-triple)" && \
    cargo_target="$(printf '%s' "$target" | tr '[:lower:]-' '[:upper:]_')" && \
    cc_target="$(printf '%s' "$target" | tr '-' '_')" && \
    export "CARGO_TARGET_${cargo_target}_LINKER=${target}-clang" && \
    export "CC_${cc_target}=${target}-clang" && \
    dx build --locked --release --debug-symbols false \
        @server --target "$target" --no-default-features --features server && \
    xx-verify ./target/dx/observable-cafe/release/web/server


FROM debian:13.6-slim@sha256:020c0d20b9880058cbe785a9db107156c3c75c2ac944a6aa7ab59f2add76a7bd

WORKDIR /app

COPY --from=builder /usr/src/app/target/dx/observable-cafe/release/web/ ./

ENV IP=0.0.0.0
ENV PORT=8080

EXPOSE 8080

USER 65532:65532

CMD ["./server"]
