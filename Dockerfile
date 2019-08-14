FROM archlinux/base
RUN pacman -Syu --noconfirm rust cargo gcc
RUN USER=root cargo new --bin pacman-repo
WORKDIR /pacman-repo
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release && rm src/*.rs
COPY . .
RUN touch src/main.rs && cargo build --release

FROM archlinux/base
COPY --from=0 /pacman-repo/target/release/pacman-repo /pacman-repo
CMD ["./pacman-repo"]
