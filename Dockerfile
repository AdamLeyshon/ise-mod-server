FROM rust:latest as builder
RUN apt-get update && apt-get install --no-install-recommends -y libpq-dev pkg-config && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/deepfreeze
ADD ./src        ./src/
ADD ./macros     ./macros/
ADD ./migrations ./migrations/
ADD ./proto      ./proto/
ADD .env.sample  ./.env
COPY ["build.rs", "Cargo.toml", "Cargo.lock", "diesel.toml", "./"]
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get install --no-install-recommends -y libpq-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /deepfreeze
COPY --from=builder /usr/src/deepfreeze/target/release/deepfreeze .
COPY --from=builder /usr/src/deepfreeze/target/release/configure .
COPY --from=builder /usr/src/deepfreeze/target/release/isebot .
COPY ./deploy/wait-for-it.sh .
COPY ./config/default.yml ./config/default.yml
RUN chmod +x ./wait-for-it.sh ./deepfreeze ./configure ./isebot

CMD ["./deepfreeze"]
