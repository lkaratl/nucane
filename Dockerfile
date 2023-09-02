FROM ubuntu:23.10

ARG EXECUTABLE_FILE
ENV EXECUTABLE_FILE=$EXECUTABLE_FILE

WORKDIR /app

COPY /target/release/$EXECUTABLE_FILE .
RUN chmod +x ./$EXECUTABLE_FILE

ENTRYPOINT ./$EXECUTABLE_FILE

# docker run --rm --network="host" --name registry registry:latest
