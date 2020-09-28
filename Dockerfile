FROM mcr.microsoft.com/windows/iotcore:1809

COPY ./target/debug/discordia.exe /tmp/discordia.exe
CMD ["./tmp/discordia.exe"]