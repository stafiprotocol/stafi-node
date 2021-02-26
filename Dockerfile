FROM phusion/baseimage:0.11 as builder
LABEL maintainer="technical@stafi.io"
LABEL description="This is the build stage for stafi. Here we create the binary."

ARG PROFILE=release
WORKDIR /stafi

COPY ./stafi /usr/local/bin

RUN apt-get update && \
        apt-get dist-upgrade -y && \
        apt-get install -y cmake curl pkg-config libssl-dev git clang && \
        #mv /usr/share/ca* /tmp && \
        #rm -rf /usr/share/*  && \
        #mv /tmp/ca-certificates /usr/share/ && \
        mkdir -p /root/.local/share/stafi && \
        ln -s /root/.local/share/stafi /data




# checks
RUN ldd /usr/local/bin/stafi && \
        /usr/local/bin/stafi --version
        #rm -rf /usr/lib/python* && \
        #rm -rf /usr/bin /usr/sbin /usr/share/man

EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/stafi", "--execution=NativeElseWasm"]
