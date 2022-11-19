#! /bin/sh

mkdir ./certs
openssl req -new -x509 -days 365 -nodes -out ./certs/server.crt -keyout ./certs/server.key -subj "${RUST_TEST_CERT_SUBJ}/CN=${RUST_TEST_HOSTNAME}" -addext "subjectAltName = DNS:*.${RUST_TEST_HOSTNAME}"
./interactsh-server -d $RUST_TEST_HOSTNAME -cert ./certs/server.crt -privkey ./certs/server.key -se -t $RUST_TEST_AUTH_TOKEN
