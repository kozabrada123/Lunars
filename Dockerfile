# copied from the rust docker refrence, lmao
FROM rust:1.67

COPY . .

RUN cargo install --path .

CMD ["lunars"]
