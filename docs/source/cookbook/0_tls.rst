TLS
===

This section describes how to generate self-signed certificates and CLRs using `OpenSSL <https://www.openssl.org/>`_. Provided instructions specifies the minimum required information only. For production usage see OpenSSL documentation.

CA
--

To generate a CA certificate with 365 days to certify with a simple subject name

.. code-block:: bash

    CA_DIR="$(pwd)/ca"

    mkdir "${CA_DIR}"

    cd "${CA_DIR}"

    mkdir certs crl

    touch index.txt

    echo 01 > serial

    echo 1000 > crlnumber

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
    countryName	           = optional
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
    " > ca.conf

    openssl genpkey -algorithm RSA -out ca.key

    openssl req -new -x509 -days 365  -config ca.conf -key ca.key -out ca.crt -subj "/CN=ca.example.com"

Certificate key file: ca.key

Certificate file: ca.crt

Server
------

To generate a server certificate signed by CA with a simple subject name and IP (both ``127.0.0.1`` and ``192.168.0.100``) subject alternative name

.. code-block:: bash

    openssl genpkey -algorithm RSA -out certs/server.key

    openssl req -new -key certs/server.key -out certs/server.csr -subj "/CN=server.example.com"

    openssl ca -config ca.conf -in certs/server.csr -out certs/server.crt -extfile <(echo 'basicConstraints=CA:FALSE
    nsComment="OpenSSL Generated Certificate"
    subjectKeyIdentifier=hash
    authorityKeyIdentifier=keyid,issuer
    keyUsage=critical,digitalSignature,keyEncipherment
    extendedKeyUsage=serverAuth
    subjectAltName=IP:127.0.0.1,IP:192.168.0.100')

To generate a server certificate signed by CA with a simple subject name and DNS (``server.example.com``) subject alternative name

.. code-block:: bash

    openssl genpkey -algorithm RSA -out server.key

    openssl req -new -key server.key -out server.csr -subj "/CN=server.example.com"

    openssl ca -config ca.conf -in certs/server.csr -out certs/server.crt -extfile <(echo 'basicConstraints=CA:FALSE
    nsComment="OpenSSL Generated Certificate"
    subjectKeyIdentifier=hash
    authorityKeyIdentifier=keyid,issuer
    keyUsage=critical,digitalSignature,keyEncipherment
    extendedKeyUsage=serverAuth
    subjectAltName=DNS:server.example.com')

Certificate key file: certs/server.key

Certificate file: certs/server.crt

Client
------

To generate a client certificate signed by CA with a simple subject name

.. code-block:: bash

    openssl genpkey -algorithm RSA -out certs/client.key

    openssl req -new -key certs/client.key -out certs/client.csr -subj "/CN=client.example.com"

    openssl ca -config ca.conf -in certs/client.csr -out certs/client.crt -extfile <(echo 'basicConstraints=CA:FALSE
    nsComment="OpenSSL Generated Certificate"
    subjectKeyIdentifier=hash
    keyUsage=critical,nonRepudiation,digitalSignature,keyEncipherment
    extendedKeyUsage=clientAuth
    authorityKeyIdentifier=keyid,issuer')

Certificate key file: certs/client.key

Certificate file: certs/client.crt

X509 lookup hash dir
--------------------

To prepare certificates signed by CA for `X509_LOOKUP_hash_dir method <https://www.openssl.org/docs/man1.1.1/man3/X509_LOOKUP_hash_dir.html>`__ in ``certs/client`` directory

.. code-block:: bash

    mkdir certs/client

    CA_HASH=$(openssl x509 -in ca.crt -subject_hash -noout)

    cp ca.crt "certs/client/$CA_HASH.0"

    openssl ca -config ca.conf -gencrl -out crl/ca.crl

    CRL_HASH=$(openssl crl -in crl/ca.crl -hash -noout)

    cp crl/ca.crl "certs/client/$CRL_HASH.r0"

A filename has the form ``hash.N`` for a certificate and the form ``hash.rN`` for a CRL where N is a sequence number that starts at zero, and is incremented consecutively for each certificate or CRL with the same hash value.

CRL
---

To revoke a client certificate signed by CA

.. code-block:: bash

    openssl ca -config ca.conf -revoke certs/client.crt

    openssl ca -config ca.conf -gencrl -out crl/ca.crl

    CRL_HASH=$(openssl crl -in crl/ca.crl -hash -noout)

    cp crl/ca.crl "certs/client/$CRL_HASH.r1"

⚠️ The sequence number N in the filename of the form ``hash.rN`` must be increased each time.
