FROM phusion/baseimage:0.11 as builder
LABEL maintainer="technical@stafi.io"
LABEL description="This is the build stage for stafi. Here we create the binary."

ARG PROFILE=release
WORKDIR /stafi

COPY . /stafi

RUN apt-get update && \
	apt-get dist-upgrade -y && \
	apt-get install -y cmake pkg-config libssl-dev git clang

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH="$PATH:$HOME/.cargo/bin" && \
	rustup toolchain install nightly && \
	rustup target add wasm32-unknown-unknown --toolchain nightly && \
	cargo install --git https://github.com/alexcrichton/wasm-gc && \
	rustup default nightly && \
	cargo build "--$PROFILE"

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL maintainer="technical@stafi.io"
LABEL description="This is the 2nd stage: a very small image where we copy the stafi binary."
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	mkdir -p /root/.local/share/stafi && \
	ln -s /root/.local/share/stafi /data

COPY --from=builder /stafi/target/$PROFILE/stafi /usr/local/bin

# checks
RUN ldd /usr/local/bin/stafi && \
	/usr/local/bin/stafi --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /usr/bin /usr/sbin /usr/share/man

EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/stafi", "--dev"]