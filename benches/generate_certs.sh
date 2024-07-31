#!/bin/bash

# prepare directories and configs for private CA
CA_DIR="$(pwd)/ca"

mkdir "${CA_DIR}" "${CA_DIR}/certs" "${CA_DIR}/crl" "${CA_DIR}/lookup-hash-dir"

touch "${CA_DIR}/index.txt"

echo 01 > "${CA_DIR}/serial"

echo 1000 > "${CA_DIR}/crlnumber"

echo "[ ca ]

default_ca      = CA_default

[ CA_default ]

dir             = ${CA_DIR}
certificate     = \$dir/ca.crt
private_key     = \$dir/ca.key
database        = \$dir/index.txt
new_certs_dir   = \$dir/certs
serial          = \$dir/serial
crl_dir         = \$dir/crl
crl             = \$dir/crl/ca.crl
crlnumber       = \$dir/crlnumber

x509_extensions = v3_ca
crl_extensions  = crl_ext

name_opt        = ca_default
cert_opt        = ca_default

default_days     = 365
default_crl_days = 30
default_md       = default
preserve         = no
policy           = policy_any

[ policy_any ]
countryName            = optional
stateOrProvinceName    = optional
organizationName       = optional
organizationalUnitName = optional
commonName             = supplied
emailAddress           = optional

####################################################################

[ req ]
default_bits       = 2048
default_keyfile    = privkey.pem
distinguished_name = req_distinguished_name
attributes         = req_attributes
x509_extensions    = v3_ca

[ req_distinguished_name ]
countryName                    = Country Name (2 letter code)
countryName_default            = US
countryName_min                = 2
countryName_max                = 2
stateOrProvinceName            = State or Province Name (full name)
stateOrProvinceName_default    =
localityName                   = Locality Name (eg, city)
localityName+default           =
0.organizationName             = Organization Name (eg, company)
0.organizationName_default     =
organizationalUnitName         = Organizational Unit Name (eg, section)
organizationalUnitName_default =
commonName                     = Common Name (e.g. server FQDN or YOUR name)
commonName_max                 = 64
emailAddress                   = Email Address
emailAddress_max               = 64

[ req_attributes ]
challengePassword     = A challenge password
challengePassword_min = 4
challengePassword_max = 20
unstructuredName      = An optional company name

[ v3_req ]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment

[ v3_ca ]
subjectKeyIdentifier=hash
authorityKeyIdentifier=keyid:always,issuer
basicConstraints = critical,CA:true
keyUsage = critical, digitalSignature, cRLSign, keyCertSign

[ crl_ext ]
authorityKeyIdentifier=keyid:always
" > "${CA_DIR}/ca.conf"

# generate a private CA key
openssl genpkey -algorithm RSA -out "${CA_DIR}/ca.key"

chmod 644 "${CA_DIR}/ca.key"

# generate a private CA certificate
openssl req -new -x509 -days 365 -config "${CA_DIR}/ca.conf" -key "${CA_DIR}/ca.key" -out "${CA_DIR}/ca.crt" -subj "/CN=media-gateway-ca"

# generate a server key
openssl genpkey -algorithm RSA -out "${CA_DIR}/certs/server.key"

chmod 644 "${CA_DIR}/certs/server.key"

# generate a server CRS
openssl req -new -key "${CA_DIR}/certs/server.key" -out "${CA_DIR}/certs/server.csr" -subj "/CN=media-gateway-server"

# generate a server certificate with DNS subject alternative name signed by the private CA
openssl ca -config "${CA_DIR}/ca.conf" -batch -in "${CA_DIR}/certs/server.csr" -out "${CA_DIR}/certs/server.crt" -extfile <(echo 'basicConstraints=CA:FALSE
nsComment="OpenSSL Generated Certificate"
subjectKeyIdentifier=hash
authorityKeyIdentifier=keyid,issuer
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName=DNS:media-gateway-server')

# generate a nginx key
openssl genpkey -algorithm RSA -out "${CA_DIR}/certs/nginx.key"

chmod 644 "${CA_DIR}/certs/nginx.key"

# generate a nginx CRS
openssl req -new -key "${CA_DIR}/certs/nginx.key" -out "${CA_DIR}/certs/nginx.csr" -subj "/CN=media-gateway-nginx"

# generate a nginx certificate with DNS subject alternative name signed by the private CA
openssl ca -config "${CA_DIR}/ca.conf" -batch -in "${CA_DIR}/certs/nginx.csr" -out "${CA_DIR}/certs/nginx.crt" -extfile <(echo 'basicConstraints=CA:FALSE
nsComment="OpenSSL Generated Certificate"
subjectKeyIdentifier=hash
authorityKeyIdentifier=keyid,issuer
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName=DNS:nginx')

# generate a client key
openssl genpkey -algorithm RSA -out "${CA_DIR}/certs/client.key"

chmod 644 "${CA_DIR}/certs/client.key"

# generate a client CRS
openssl req -new -key "${CA_DIR}/certs/client.key" -out "${CA_DIR}/certs/client.csr" -subj "/CN=media-gateway-client"

# generate a client certificate signed by the private CA
openssl ca -config "${CA_DIR}/ca.conf" -batch -in "${CA_DIR}/certs/client.csr" -out "${CA_DIR}/certs/client.crt" -extfile <(echo 'basicConstraints=CA:FALSE
nsComment="OpenSSL Generated Certificate"
subjectKeyIdentifier=hash
keyUsage=critical,nonRepudiation,digitalSignature,keyEncipherment
extendedKeyUsage=clientAuth
authorityKeyIdentifier=keyid,issuer')

# add the private CA certificate to the directory to be used for X509_LOOKUP_hash_dir method
CA_HASH=$(openssl x509 -in "${CA_DIR}/ca.crt" -subject_hash -noout)

cp "${CA_DIR}/ca.crt" "${CA_DIR}/lookup-hash-dir/$CA_HASH.0"

# generate a CRL for the private CA
openssl ca -config "${CA_DIR}/ca.conf" -gencrl -out "${CA_DIR}/crl/ca.crl"

# add the private CA CRL to the directory to be used for X509_LOOKUP_hash_dir method
CRL_HASH=$(openssl crl -in "${CA_DIR}/crl/ca.crl" -hash -noout)

cp "${CA_DIR}/crl/ca.crl" "${CA_DIR}/lookup-hash-dir/$CRL_HASH.r0"
