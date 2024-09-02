# copied from the rust docker refrence, lmao
FROM rust:1.75

WORKDIR /usr/src/lunars

# https://stackoverflow.com/questions/58473606/cache-rust-dependencies-with-docker-build
COPY Cargo.toml .
COPY Cargo.lock .
RUN echo "fn main() {}" > dummy.rs

RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

COPY . .

RUN cargo build --release

CMD ["target/release/lunars"]
