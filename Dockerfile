# -------- Build Stage --------
# Switch to the nightly toolchain to get support for the unstable 2024 edition
# As per user's suggestion, use a smaller slim image for optimization
FROM rust:1.85-slim-bullseye AS builder
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev && rm -rf /var/lib/apt/lists/*
COPY . .
# We remove --locked to allow Cargo to update the lock file inside the container
RUN cargo build --release
# 在构建阶段创建数据目录，以便复制到运行时镜像
RUN mkdir -p /usr/src/app/data

# -------- Runtime Stage --------
FROM gcr.io/distroless/cc AS runtime
LABEL maintainer="atlassian-rust"

# Timezone & SSL certs (optional)
# COPY --from=builder /usr/share/zoneinfo /usr/share/zoneinfo
# COPY --from=builder /etc/ssl/certs /etc/ssl/certs

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/atlassian-rust-docker .
# 复制模板与静态资源
COPY --from=builder /usr/src/app/templates ./templates
COPY --from=builder /usr/src/app/static ./static
COPY --from=builder /usr/src/app/.env .
# 复制数据目录
COPY --from=builder /usr/src/app/data ./data

EXPOSE 8080
ENTRYPOINT ["/app/atlassian-rust-docker"]
