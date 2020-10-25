#!/usr/bin/env sh

set -e

# some openssl tooling
touch index.txt
echo '01' > serial.txt

# create CA certificate
openssl req -x509 -batch -config openssl-ca.cnf -newkey rsa:2048 -sha256 -nodes -out ca.cert -outform PEM

# create server key & signing request
openssl req -batch -config openssl-server.cnf -newkey rsa:2048 -sha256 -nodes -out server.csr -outform PEM

# sign the signing request with the CA key
openssl ca -batch -notext -config openssl-ca.cnf -policy signing_policy -extensions signing_req -out server_tmp.cert -infiles server.csr

cat server_tmp.cert > server.cert
cat ca.cert >> server.cert
rm server_tmp.cert 01.pem
