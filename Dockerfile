FROM archlinux/base
RUN pacman -Syu --noconfirm rust cargo gcc
RUN USER=root cargo new --bin app
WORKDIR /app
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release && rm src/*.rs
COPY . .
RUN cargo build --release
CMD ["target/release/pacman-repo"]
