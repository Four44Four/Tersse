FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
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

RUN sed -i 's/\r$//' docker/run-valgrind-tests.sh \
    && chmod +x docker/run-valgrind-tests.sh \
    && cargo test --tests --no-run

CMD ["docker/run-valgrind-tests.sh"]
