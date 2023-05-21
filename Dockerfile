FROM rust:latest

RUN apt-get update && apt-get install -y \
    libgtk-3-dev \
    pulseaudio \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN cargo build --release --locked

COPY src/ ./src/
COPY tests/ ./tests/

RUN cargo build --release --locked

ENV PULSE_SERVER unix:/run/user/1000/pulse/native

ENTRYPOINT ["cargo", "test", "--release"]
