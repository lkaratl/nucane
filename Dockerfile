FROM ubuntu:23.10

RUN apt-get update
RUN apt-get install -y openssl
RUN apt-get install -y ca-certificates

ARG EXECUTABLE_FILE
ENV EXECUTABLE_FILE=$EXECUTABLE_FILE

WORKDIR /app

COPY /target/release/$EXECUTABLE_FILE .
RUN chmod +x ./$EXECUTABLE_FILE

ENTRYPOINT ./$EXECUTABLE_FILE
