FROM rust

RUN apt update && apt upgrade -y

RUN groupadd -r user && useradd -r -g user user
WORKDIR /home/user

RUN git clone https://github.com/smallkirby/rapt.git
RUN cd rapt && cargo build