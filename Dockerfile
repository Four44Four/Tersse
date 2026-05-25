FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        bsdutils \
        libncurses-dev \
        pkg-config \
        valgrind \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal

WORKDIR /app

COPY . .

RUN sed -i 's/\r$//' docker/entrypoint.sh docker/run-valgrind-tests.sh docker/run-valgrind-examples.sh \
    && chmod +x docker/entrypoint.sh docker/run-valgrind-tests.sh docker/run-valgrind-examples.sh \
    && cargo test --tests --no-run --features test-api \
    && cargo build --bins

ENTRYPOINT ["/app/docker/entrypoint.sh"]
CMD ["tests"]
