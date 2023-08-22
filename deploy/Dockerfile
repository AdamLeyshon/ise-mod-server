FROM rust:buster as planner
WORKDIR chef
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust:buster as cacher
WORKDIR /chef
RUN cargo install cargo-chef
COPY --from=planner /chef/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:buster as builder
RUN apt-get update && apt-get install --no-install-recommends -y libpq-dev pkg-config && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/deepfreeze
ADD ./src        ./src/
ADD ./macros     ./macros/
ADD ./migrations ./migrations/
ADD ./proto      ./proto/
ADD .env.sample  ./.env
COPY ["build.rs", "Cargo.toml", "Cargo.lock", "diesel.toml", "./"]
RUN ls -lah
COPY --from=cacher /chef/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get install --no-install-recommends -y libpq5 && rm -rf /var/lib/apt/lists/*
WORKDIR /deepfreeze
COPY --from=builder /usr/src/deepfreeze/target/release/deepfreeze .
COPY --from=builder /usr/src/deepfreeze/target/release/configure .
COPY --from=builder /usr/src/deepfreeze/target/release/isebot .
COPY ./deploy/wait-for-it.sh .
COPY ./config/default.yml ./config/default.yml
RUN chmod +x ./wait-for-it.sh ./deepfreeze ./configure ./isebot

CMD ["./deepfreeze"]
