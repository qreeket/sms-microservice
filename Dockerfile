## Build: docker build -t quabynah1809/qreeket-sms:latest .
FROM rust:1.69-slim-buster AS builder

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY . .

# install openssl
RUN apt-get update && apt-get install -y libssl-dev pkg-config

# this build step will cache your dependencies
RUN cargo build --release --target x86_64-unknown-linux-musl --locked

# build for release
#RUN rm ./target/release/deps/sms*
#RUN cargo build --release


# our final base
FROM rust:1.69.0-alpine3.17
WORKDIR /
# copy the build artifact from the build stage
COPY --from=builder /sms/target/release/sms .

EXPOSE 9903

# set the startup command to run your binary
CMD ["sms"]

