# Dockerfile for creating a statically-linked Rust application using docker's
# multi-stage build feature. This also leverages the docker build cache to avoid
# re-downloading dependencies if they have not changed.
FROM rust:1.44 AS build
WORKDIR /usr/src/ul-cms
COPY . .
RUN cargo install --path .


FROM alpine:latest
RUN apk update 
WORKDIR /usr/scr/ul-cms
#&& apk add -y extra-runtime-dependencies
COPY --from=build /usr/src/ul-cms .

CMD ul-cms