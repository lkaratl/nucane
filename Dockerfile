FROM ubuntu:23.10

ARG EXECUTABLE_NAME=""

COPY /target/release/${EXECUTABLE_NAME} /

ENTRYPOINT ["./${EXECUTABLE_NAME}"]

# docker run --rm --network host --name registry registry:latest
