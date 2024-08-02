TLS
===

Media Gateway TLS features:

* HTTPS
* client certificate authentication

TLS features are implemented using `OpenSSL <https://www.openssl.org/>`__.

HTTPS
-----

Media Gateway supports both self-signed and signed by CA server certificates. Certificates should be in PEM format. To enable HTTPS in Media Gateway update both server and client configuration.

The protocol in ``url`` field in the client configuration must be updated to ``https``.

Self-signed server certificates
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

``server.crt`` is a file with the server certificate in PEM format.

``server.key`` is a file with the server key in PEM format.

.. code-block:: json
    :caption: server

    "tls": {
        "identity": {
            "certificate": "server.crt",
            "key": "server.key"
        }
    }

.. code-block:: json
    :caption: client

    "tls": {
        "root_certificate": "server.crt"
    }

Signed by a private CA server certificates
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

``server.crt`` is a file with the server certificate in PEM format.

``server.key`` is a file with the server key in PEM format.

``ca.crt`` is a file with the CA certificate in PEM format.

.. code-block:: json
    :caption: server

    "tls": {
        "identity": {
            "certificate": "server.crt",
            "key": "server.key"
        }
    }

.. code-block:: json
    :caption: client

    "tls": {
        "root_certificate": "ca.crt"
    }

Signed by a public CA server certificates
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

``server.crt`` is a file with a sequence of certificates, the first being the leaf certificate, and the remainder forming the chain of certificates up to and including the trusted root certificate.

``server.key`` is a file with the server key in PEM format.

.. code-block:: json
    :caption: server

    "tls": {
        "identity": {
            "certificate": "server.crt",
            "key": "server.key"
        }
    }

Client certificate authentication
---------------------------------

Client certificate authentication is an optional feature in Media Gateway. Only signed by CA client certificates can be used. Certificates should be in PEM format.

The server uses a store with trusted X509 certificates to verify peer certificates. The store automatically (without a server restart) loads certificates and CRLs from the specified directory. Certificates and CRLs should be added to the directory in accordance with `X509_LOOKUP_hash_dir method <https://www.openssl.org/docs/man1.1.1/man3/X509_LOOKUP_hash_dir.html>`__ requirements. For each certificate at least one CRL must be in the directory. CRL might be without revoked certificates. A new CRL must be loaded when the previous CRL is expired.

``ca.crt`` is a file with the CA certificate in PEM format.

``ca.crl`` is a file with CRL in PEM format.

``/opt/etc/certs/lookup-hash-dir`` is a directory with CA certificates and CRLs.

To add a new certificate and corresponding CRL

.. code-block:: bash

    CA_HASH=$(openssl x509 -in ca.crt -subject_hash -noout)

    cp ca.crt "/opt/etc/certs/lookup-hash-dir/$CA_HASH.0"

    CRL_HASH=$(openssl crl -in ca.crl -hash -noout)

    cp ca.crl "/opt/etc/certs/lookup-hash-dir/$CRL_HASH.r0"

To enable client certificate authentication in Media Gateway update both server and client configuration.

``/opt/etc/certs/lookup-hash-dir`` is a directory with CA certificates and CRLs.

``client.crt`` is a file with a client certificate in PEM format.

``client.key`` is a file with a PEM encoded PKCS #8 formatted client key.

.. code-block:: json
    :caption: server

    "tls": {
        // see HTTPS section
        "peers": {
            "lookup_hash_directory" : "/opt/etc/certs/lookup-hash-dir",
            "crl_enabled": true
        }
    }

.. code-block:: json
    :caption: client

    "tls": {
        // see HTTPS section
        "identity": {
            "certificate": "client.crt",
            "key": "client.key"
        }
    }

Certificate generation with a private CA
----------------------------------------

This section describes how to generate certificates and CLRs signed by a private CA using `OpenSSL <https://www.openssl.org/>`_. Provided instructions specifies the minimum required information only. For production usage see OpenSSL documentation.

CA
^^

To set up a private CA and generate a certificate

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

``ca.crt`` is a file with CA certificate in PEM format.

``ca.key`` is a file with CA key in PEM format.

Server
^^^^^^

To generate a server certificate signed by the CA with a simple subject name and IP (both ``127.0.0.1`` and ``192.168.0.100``) subject alternative name

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

    openssl genpkey -algorithm RSA -out certs/server.key

    openssl req -new -key certs/server.key -out certs/server.csr -subj "/CN=server.example.com"

    openssl ca -config ca.conf -in certs/server.csr -out certs/server.crt -extfile <(echo 'basicConstraints=CA:FALSE
    nsComment="OpenSSL Generated Certificate"
    subjectKeyIdentifier=hash
    authorityKeyIdentifier=keyid,issuer
    keyUsage=critical,digitalSignature,keyEncipherment
    extendedKeyUsage=serverAuth
    subjectAltName=DNS:server.example.com')

``certs/server.crt`` is a file with a server certificate in PEM format.

``certs/server.key`` is a file with a server key in PEM format.

Client
------

To generate a client certificate signed by the CA with a simple subject name

.. code-block:: bash

    openssl genpkey -algorithm RSA -out certs/client.key

    openssl req -new -key certs/client.key -out certs/client.csr -subj "/CN=client.example.com"

    openssl ca -config ca.conf -in certs/client.csr -out certs/client.crt -extfile <(echo 'basicConstraints=CA:FALSE
    nsComment="OpenSSL Generated Certificate"
    subjectKeyIdentifier=hash
    keyUsage=critical,nonRepudiation,digitalSignature,keyEncipherment
    extendedKeyUsage=clientAuth
    authorityKeyIdentifier=keyid,issuer')

``certs/client.crt`` is a file with a client certificate in PEM format.

``certs/client.key`` is a file with a client key in PEM format.

X509 lookup hash dir
--------------------

To prepare certificates signed by the CA for `X509_LOOKUP_hash_dir method <https://www.openssl.org/docs/man1.1.1/man3/X509_LOOKUP_hash_dir.html>`__ in ``certs/client`` directory

.. code-block:: bash

    mkdir lookup-hash-dir

    CA_HASH=$(openssl x509 -in ca.crt -subject_hash -noout)

    cp ca.crt "lookup-hash-dir/$CA_HASH.0"

    openssl ca -config ca.conf -gencrl -out crl/ca.crl

    CRL_HASH=$(openssl crl -in crl/ca.crl -hash -noout)

    cp crl/ca.crl "lookup-hash-dir/$CRL_HASH.r0"

A filename has the form ``hash.N`` for a certificate and the form ``hash.rN`` for a CRL where N is a sequence number that starts at zero, and is incremented consecutively for each certificate or CRL with the same hash value.

CRL
---

To revoke a client certificate signed by the CA

.. code-block:: bash

    openssl ca -config ca.conf -revoke certs/client.crt

    openssl ca -config ca.conf -gencrl -out crl/ca.crl

    CRL_HASH=$(openssl crl -in crl/ca.crl -hash -noout)

    cp crl/ca.crl "lookup-hash-dir/$CRL_HASH.r1"

⚠️ The sequence number N in the filename of the form ``hash.rN`` must be increased each time.
