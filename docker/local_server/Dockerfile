FROM golang:1.19.2-bullseye
WORKDIR /app
ENV GOBIN=/app
RUN apt-get update && apt-get install -y python3-impacket
RUN go install -v github.com/projectdiscovery/interactsh/cmd/interactsh-server@latest
COPY ./server_start.sh /app/server_start.sh
RUN chmod +x ./server_start.sh
CMD ./server_start.sh
EXPOSE 80/tcp
EXPOSE 443/tcp
EXPOSE 53/tcp
EXPOSE 53/udp
EXPOSE 25/tcp
EXPOSE 389/tcp
EXPOSE 587/tcp
EXPOSE 21/tcp
EXPOSE 445/tcp
