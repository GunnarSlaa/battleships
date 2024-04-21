FROM ubuntu
RUN apt-get update && apt-get install -y libssl-dev openssl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY ./target/debug/rustserver /usr/local/bin/rustserver
CMD ["rustserver"]