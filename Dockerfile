FROM rust:1.76-bookworm AS build

WORKDIR /app

# Copy all the source files including config.toml
COPY . /app

RUN apt-get update && apt-get install -y libssl-dev pkg-config
RUN cargo build --profile maxperf

FROM debian:bookworm

RUN mkdir /app
RUN apt-get update && apt-get install -y openssl ca-certificates

# Copy the built binary from the build stage
COPY --from=build /app/target/maxperf/trident /app/trident

COPY --from=build /app/example.config.toml /app/example.config.toml


# Copy the spin script
COPY spin.sh /app/spin.sh

WORKDIR /app

# Expose the application port
EXPOSE 3001

ENTRYPOINT ["sh", "spin.sh"]