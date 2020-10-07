FROM ubuntu:focal
COPY ./target/debug/discordia /bin/discordia

ENV DEBIAN_FRONTEND=noninteractive
RUN apt update && apt install -y ffmpeg youtube-dl
RUN /bin/discordia
