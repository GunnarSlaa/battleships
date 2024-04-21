FROM ubuntu
RUN apt-get update && apt-get install -y libssl-dev openssl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY ./target/debug/rustbot /usr/local/bin/rustbot
CMD ["rustbot"]