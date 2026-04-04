FROM rust:1.80-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev pkgconfig postgresql-dev

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static

RUN cargo build --release

FROM alpine:3.19

RUN apk add --no-cache postgresql-client tzdata

ENV TZ=America/Argentina/Buenos_Aires
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

WORKDIR /app

COPY --from=builder /build/target/release/bkpm /app/bkpm
COPY --from=builder /build/static /app/static

RUN mkdir -p /backups /data

ENV BACKUP_DIR=/backups
ENV DB_PATH=/data/bkpm.db
ENV PORT=3450
ENV HTML_PATH=/app/static/index.html

EXPOSE 3450

CMD ["/app/bkpm"]
