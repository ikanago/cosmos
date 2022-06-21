ARG VARIANT=latest
FROM ghcr.io/sarisia/mikanos:${VARIANT}

USER vscode

ENV PATH=/home/vscode/.cargo/bin:$PATH

SHELL ["/bin/bash", "-oeux", "pipefail", "-c"]

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y

RUN mkdir -p /home/vscode/.vscode-server/extensions \
    && chown -R vscode /home/vscode/.vscode-server

ENV CARGO_MAKE_VERSION='0.35.13'
ENV CARGO_MAKE_BIN="cargo-make-v${CARGO_MAKE_VERSION}-x86_64-unknown-linux-musl"
ENV CARGO_MAKE_ZIP="${CARGO_MAKE_BIN}.zip"

RUN wget -q "https://github.com/sagiegurari/cargo-make/releases/download/${CARGO_MAKE_VERSION}/${CARGO_MAKE_ZIP}" \
    && unzip "${CARGO_MAKE_ZIP}" \
    && mkdir -p "${HOME}/.local/bin" \
    && ls -la \
    && cp ${CARGO_MAKE_BIN}/{cargo-make,makers} "${HOME}/.local/bin" \
    && rm -rf "${CARGO_MAKE_BIN}" \
    && rm -f "${CARGO_MAKE_ZIP}"
