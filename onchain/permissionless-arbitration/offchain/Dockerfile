FROM --platform=linux/amd64 diegonehab/machine-emulator
RUN apt-get -y update; apt-get -y install curl
RUN curl -sSL https://github.com/foundry-rs/foundry/releases/download/nightly/foundry_nightly_linux_$(dpkg --print-architecture).tar.gz | \
    tar -zx -C /usr/local/bin

ADD ./ ./src/
WORKDIR "./src"

RUN chmod +x ./entrypoint.lua
ENTRYPOINT ["./entrypoint.lua"]
