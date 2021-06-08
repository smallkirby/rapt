FROM ubuntu:20.04
SHELL ["/bin/bash", "-c"]

RUN apt update && apt upgrade -y ca-certificates apt-utils curl wget build-essential binutils gcc git libssl-dev libssl1.1 autoconf automake
#RUN apt install pkg-config -y

RUN groupadd -r user && useradd -r -g user user
WORKDIR /home/user

ENV RUST_VERSION stable
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH $PATH:$HOME/.cargo/bin:/usr/local/cargo/bin

RUN git clone https://github.com/smallkirby/rapt.git
#RUN source $HOME/.cargo/env && cd rapt && source $HOME/.bashrc && bash -c "cargo build"
RUN cd rapt && mkdir lists