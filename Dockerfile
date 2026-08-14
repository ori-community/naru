FROM rust:alpine AS build

WORKDIR /app
COPY . /app
RUN apk add --no-cache musl-dev openssl-libs-static openssl-dev && \
    cargo build --release


FROM gcr.io/distroless/static-debian13

WORKDIR /app

COPY --from=build /app/target/release/naru /app/naru

ENTRYPOINT ["/app/naru"]
CMD ["-c", "/config.yaml"]
