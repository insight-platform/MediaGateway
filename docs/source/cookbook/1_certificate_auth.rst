Client certificate authentication
=================================

Client certificate authentication is an optional feature in Media Gateway and can be used only if HTTPS is enabled (see :doc:`0_https`). Only signed by CA client certificates can be used. Certificates should be in PEM format. CRLs are supported but they are not mandatory.

The server uses a store with trusted X509 certificates and CRLs to verify peer certificates. The store automatically (without a server restart) loads certificates and CRLs from the specified directory. Certificates and CRLs should be added to the directory in accordance with `X509_LOOKUP_hash_dir method <https://www.openssl.org/docs/man1.1.1/man3/X509_LOOKUP_hash_dir.html>`__ requirements. If CRLs are enabled for each certificate at least one CRL must be in the directory. The CRL may contain no revoked certificates. A new CRL must be added when the previous CRL is expired.

Prerequisites
-------------

* Docker
* Docker Compose
* openssl
* curl

Configuring Media Gateway
-------------------------

To enable client certificate authentication in Media Gateway both server and client configuration should be updated.

.. code-block:: json
    :caption: server with enabled CRLs

    "tls": {
        // see HTTPS
        "peers": {
            "lookup_hash_directory" : "/etc/certs/lookup-hash-dir",
            "crl_enabled": true
        }
    }

.. code-block:: json
    :caption: server with disabled CRLs

    "tls": {
        // see HTTPS
        "peers": {
            "lookup_hash_directory" : "/etc/certs/lookup-hash-dir",
            "crl_enabled": false
        }
    }

.. code-block:: json
    :caption: client

    "tls": {
        // see HTTPS
        "identity": {
            "certificate": "client.crt",
            "key": "client.key"
        }
    }

where

* ``/etc/certs/lookup-hash-dir`` is a directory with CA certificates and CRLs.

* ``client.crt`` is a file with a client certificate in PEM format.

* ``client.key`` is a file with a PEM encoded PKCS #8 formatted client key.

Generating certificates and CRLs
--------------------------------

This section describes how to generate certificates and CLRs signed by a private CA using `OpenSSL <https://www.openssl.org/>`_. Provided instructions specifies the minimum required information only. For production usage see OpenSSL documentation.

Creating a private CA
^^^^^^^^^^^^^^^^^^^^^

Create directories

.. code-block:: bash

    CA_DIR="$(pwd)/ca"

    mkdir "${CA_DIR}" "${CA_DIR}/certs" "${CA_DIR}/crl"

Prepare a CA database

.. code-block:: bash

    touch "${CA_DIR}/index.txt"

    echo 01 > "${CA_DIR}/serial"

    echo 1000 > "${CA_DIR}/crlnumber"

Prepare a CA configuration file

.. code-block:: bash

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
    " > "${CA_DIR}/ca.conf"

Generate a CA private key and certificate

.. code-block:: bash

    openssl genpkey -algorithm RSA -out "${CA_DIR}/ca.key"

    openssl req -new -x509 -days 365 -config "${CA_DIR}/ca.conf" -key "${CA_DIR}/ca.key" -out "${CA_DIR}/ca.crt" -subj "/CN=media-gateway-ca"

Generating a server certificate
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Generate a private key and certificate signing request

.. code-block:: bash

    openssl genpkey -algorithm RSA -out "${CA_DIR}/certs/server.key"

    openssl req -new -key "${CA_DIR}/certs/server.key" -out "${CA_DIR}/certs/server.csr" -subj "/CN=media-gateway-server"

If the client connects to the server by IP generate a certificate with IP subject alternative name. Otherwise generate a certificate with DNS subject alternative name.

In commands below replace `192.168.0.108` and `media-gateway-server` with your values.

.. code-block:: bash
    :caption: IP SAN

    export HOST_IP="192.168.0.108"

    openssl ca -config "${CA_DIR}/ca.conf" -in "${CA_DIR}/certs/server.csr" -out "${CA_DIR}/certs/server.crt" -extfile <(echo "basicConstraints=CA:FALSE
    nsComment=\"OpenSSL Generated Certificate\"
    subjectKeyIdentifier=hash
    authorityKeyIdentifier=keyid,issuer
    keyUsage=critical,digitalSignature,keyEncipherment
    extendedKeyUsage=serverAuth
    subjectAltName=IP:${HOST_IP}")

.. code-block:: bash
    :caption: DNS SAN

    export MEDIA_GATEWAY_SERVER_DNS="media-gateway-server"

    openssl ca -config "${CA_DIR}/ca.conf" -in "${CA_DIR}/certs/server.csr" -out "${CA_DIR}/certs/server.crt" -extfile <(echo "basicConstraints=CA:FALSE
    nsComment=\"OpenSSL Generated Certificate\"
    subjectKeyIdentifier=hash
    authorityKeyIdentifier=keyid,issuer
    keyUsage=critical,digitalSignature,keyEncipherment
    extendedKeyUsage=serverAuth
    subjectAltName=DNS:${MEDIA_GATEWAY_SERVER_DNS}")

Generating a client certificate
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Generate a private key, certificate signing request and a certificate

.. code-block:: bash

    openssl genpkey -algorithm RSA -out "${CA_DIR}/certs/client.key"

    openssl req -new -key "${CA_DIR}/certs/client.key" -out "${CA_DIR}/certs/client.csr" -subj "/CN=media-gateway-client"

    openssl ca -config "${CA_DIR}/ca.conf" -in "${CA_DIR}/certs/client.csr" -out "${CA_DIR}/certs/client.crt" -extfile <(echo 'basicConstraints=CA:FALSE
    nsComment="OpenSSL Generated Certificate"
    subjectKeyIdentifier=hash
    keyUsage=critical,nonRepudiation,digitalSignature,keyEncipherment
    extendedKeyUsage=clientAuth
    authorityKeyIdentifier=keyid,issuer')

Preparing X509 lookup hash directory
------------------------------------

.. admonition:: OpenSSL documentation

    X509_LOOKUP_hash_dir is a more advanced method, which loads certificates and CRLs on demand, and caches them in memory once they are loaded. As of OpenSSL 1.0.0, it also checks for newer CRLs upon each lookup, so that newer CRLs are as soon as they appear in the directory.

    The directory should contain one certificate or CRL per file in PEM format, with a filename of the form hash.N for a certificate, or hash.rN for a CRL. The hash is the value returned by the X509_NAME_hash(3) function applied to the subject name for certificates or issuer name for CRLs. The hash can also be obtained via the -hash option of the x509(1) or crl(1) commands.

    The .N or .rN suffix is a sequence number that starts at zero, and is incremented consecutively for each certificate or CRL with the same hash value. Gaps in the sequence numbers are not supported, it is assumed that there are no more objects with the same hash beyond the first missing number in the sequence.

    Sequence numbers make it possible for the directory to contain multiple certificates with same subject name hash value. For example, it is possible to have in the store several certificates with same subject or several CRLs with same issuer (and, for example, different validity period).

    When checking for new CRLs once one CRL for given hash value is loaded, hash_dir lookup method checks only for certificates with sequence number greater than that of the already cached CRL.

Create a directory

.. code-block:: bash

    mkdir "${CA_DIR}/lookup-hash-dir"

Add the CA certificate to the directory

.. code-block:: bash

    CA_HASH=$(openssl x509 -in "${CA_DIR}/ca.crt" -subject_hash -noout)

    cp "${CA_DIR}/ca.crt" "${CA_DIR}/lookup-hash-dir/$CA_HASH.0"

If CRLs are used generate and add an empty CRL

.. code-block:: bash

    openssl ca -config "${CA_DIR}/ca.conf" -gencrl -out "${CA_DIR}/crl/ca.crl"

    CRL_HASH=$(openssl crl -in "${CA_DIR}/crl/ca.crl" -hash -noout)

    cp "${CA_DIR}/crl/ca.crl" "${CA_DIR}/lookup-hash-dir/$CRL_HASH.r0"

.. _certificate revocation:

Revoking certificates
---------------------

Omit this section if CRLs are not used or Media Gateway has not been launched.

Revoke a client certificate

.. code-block:: bash

    openssl ca -config "${CA_DIR}/ca.conf" -revoke "${CA_DIR}/certs/client.crt"

Generate a new CRL and update X509 lookup hash directory.

.. warning::

    The sequence number N in the filename of the form ``hash.rN`` must be increased each time.

.. code-block:: bash

    openssl ca -config "${CA_DIR}/ca.conf" -gencrl -out "${CA_DIR}/crl/ca.crl"

    CRL_HASH=$(openssl crl -in "${CA_DIR}/crl/ca.crl" -hash -noout)

    cp "${CA_DIR}/crl/ca.crl" "${CA_DIR}/lookup-hash-dir/$CRL_HASH.r1"

Testing
-------

Server
^^^^^^

To test the server only a certificate with IP SAN is used.

Prepare the configuration file with enabled CRLs

.. code-block:: bash

    cat << EOF > media-gateway-server.json
    {
        "ip": "0.0.0.0",
        "port": 8080,
        "tls": {
            "identity": {
                "certificate": "/etc/certs/server.crt",
                "key": "/etc/certs/server.key"
            },
            "peers": {
                "lookup_hash_directory": "/etc/certs/lookup-hash-dir",
                "crl_enabled": true
            }
        },
        "out_stream": {
            "url": "pub+bind:ipc:///tmp/server",
            "send_timeout": {
                "secs": 1,
                "nanos": 0
            },
            "send_retries": 3,
            "receive_timeout": {
                "secs": 1,
                "nanos": 0
            },
            "receive_retries": 3,
            "send_hwm": 1000,
            "receive_hwm": 1000,
            "inflight_ops": 100
        }
    }
    EOF

Launch the server (change the value of ``MEDIA_GATEWAY_PORT`` in the command below if required)

.. code-block:: bash
    :caption: x86_64

    export MEDIA_GATEWAY_PORT=8080

    docker run -d \
        -v $(pwd)/media-gateway-server.json:/opt/etc/custom_config.json \
        -v ${CA_DIR}/certs/server.key:/etc/certs/server.key \
        -v ${CA_DIR}/certs/server.crt:/etc/certs/server.crt \
        -v ${CA_DIR}/lookup-hash-dir:/etc/certs/lookup-hash-dir \
        -p ${MEDIA_GATEWAY_PORT}:8080 \
        --name media-gateway-server \
        ghcr.io/insight-platform/media-gateway-server-x86:latest \
        /opt/etc/custom_config.json

.. code-block:: bash
    :caption: ARM64

    export MEDIA_GATEWAY_PORT=8080

    docker run -d \
        -v $(pwd)/media-gateway-server.json:/opt/etc/custom_config.json \
        -v ${CA_DIR}/certs/server.key:/etc/certs/server.key \
        -v ${CA_DIR}/certs/server.crt:/etc/certs/server.crt \
        -v ${CA_DIR}/lookup-hash-dir:/etc/certs/lookup-hash-dir \
        -p ${MEDIA_GATEWAY_PORT}:8080 \
        --name media-gateway-server \
        ghcr.io/insight-platform/media-gateway-server-arm64:latest \
        /opt/etc/custom_config.json

Send the request to the server

.. code-block:: bash

    curl --cacert "${CA_DIR}/ca.crt" --cert "${CA_DIR}/certs/client.crt" --key "${CA_DIR}/certs/client.key" -v https://$HOST_IP:$MEDIA_GATEWAY_PORT/health

HTTP response with ``200 OK`` status code and the body as below should be returned.

.. code-block:: json

    {"status": "healthy"}

Revoke the client certificate using :ref:`the section <certificate revocation>` and send the request to the server again. An error response with the message as below should be returned.

.. code-block::

    error:0A000414:SSL routines::sslv3 alert certificate revoked

Clean up after testing

.. code-block:: bash

    docker stop media-gateway-server

    docker rm media-gateway-server

    rm -rf ca media-gateway-server.json

e2e
^^^

To test both server and client based on :doc:`3_usage_example`

* generate a certificate with DNS SAN
* update ``server_config.json`` and ``client_config.json`` in the downloaded archive as described above and in :ref:`HTTPS <private ca https>` guide
* add volumes for ``media-gateway-client``` (key and certificate files) and ``media-gateway-server`` (key and certificate files) in ``docker-compose-x86.yaml`` and ``docker-compose-arm64.yaml``  in the downloaded archive

Clean up after testing

.. code-block:: bash

    rm -rf ca
