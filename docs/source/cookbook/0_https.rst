HTTPS
=====

This guide shows how to enable HTTPS protocol in Media Gateway. Both self-signed and signed by CA server certificates are supported. Certificates should be in PEM format.

.. important::

    The protocol in ``url`` field in the client configuration must be updated to ``https``.

Prerequisites
-------------

* Docker
* Docker Compose
* openssl
* curl

Using a self-signed certificate
-------------------------------

Configuring Media Gateway
^^^^^^^^^^^^^^^^^^^^^^^^^

To use a self-signed certificate update server and client configurations.

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

where

* ``server.crt`` is a file with the server certificate in PEM format.

* ``server.key`` is a file with the server key in PEM format.

Generating certificates
^^^^^^^^^^^^^^^^^^^^^^^

Generate a private key and certificate signing request

.. code-block:: bash

    mkdir certs

    openssl genpkey -algorithm RSA -out certs/server.key

    openssl req -new -key certs/server.key -out certs/server.csr -subj "/CN=media-gateway-server"

If the client connects to the server by IP generate a certificate with IP subject alternative name. Otherwise generate a certificate with DNS subject alternative name.

In commands below replace `192.168.0.108` and `media-gateway-server` with your values.

.. code-block:: bash
    :caption: IP SAN

    export HOST_IP="192.168.0.108"

    openssl x509 -req -days 365 -key certs/server.key -in certs/server.csr -out certs/server.crt -extfile <(echo "subjectAltName=IP:${HOST_IP}")

.. code-block:: bash
    :caption: DNS SAN

    export MEDIA_GATEWAY_SERVER_DNS="media-gateway-server"

    openssl x509 -req -days 365 -key certs/server.key -in certs/server.csr -out certs/server.crt -extfile <(echo "subjectAltName=DNS:${MEDIA_GATEWAY_SERVER_DNS}")

Testing
^^^^^^^

Server
""""""

To test the server only a certificate with IP SAN is used.

Prepare the configuration file

.. code-block:: bash

    cat << EOF > media-gateway-server.json
    {
        "ip": "0.0.0.0",
        "port": 8080,
        "tls": {
            "identity": {
                "certificate": "/etc/certs/server.crt",
                "key": "/etc/certs/server.key"
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
            "fix_ipc_permissions": 511
        }
    }
    EOF

Launch the server (change the value of `MEDIA_GATEWAY_PORT` in the command below if required)

.. code-block:: bash
    :caption: x86_64

    export MEDIA_GATEWAY_PORT=8080

    docker run -d \
        -v $(pwd)/media-gateway-server.json:/opt/etc/custom_config.json \
        -v $(pwd)/certs:/etc/certs \
        -p $MEDIA_GATEWAY_PORT:8080 \
        --name media-gateway-server \
        ghcr.io/insight-platform/media-gateway-server-x86:latest \
        /opt/etc/custom_config.json

.. code-block:: bash
    :caption: ARM64

    export MEDIA_GATEWAY_PORT=8080

    docker run -d \
        -v $(pwd)/media-gateway-server.json:/opt/etc/custom_config.json \
        -v $(pwd)/certs:/etc/certs \
        -p $MEDIA_GATEWAY_PORT:8080 \
        --name media-gateway-server \
        ghcr.io/insight-platform/media-gateway-server-arm64:latest \
        /opt/etc/custom_config.json

Send the request to the server

.. code-block:: bash

    curl --cacert certs/server.crt -v https://$HOST_IP:$MEDIA_GATEWAY_PORT/health

HTTP response with ``200 OK`` status code and the body as below should be returned.

.. code-block:: json

    {"status": "healthy"}

Clean up after testing

.. code-block:: bash

    docker stop media-gateway-server

    docker rm media-gateway-server

    rm -rf certs media-gateway-server.json

e2e
"""

To test both server and client based on :doc:`3_usage_example`

* generate a certificate with DNS SAN
* update ``server_config.json`` and ``client_config.json`` in the downloaded archive as described above
* add volumes for ``media-gateway-client``` (a certificate file) and ``media-gateway-server`` (key and certificate files) in ``docker-compose-x86.yaml`` and ``docker-compose-arm64.yaml``  in the downloaded archive

Clean up after testing

.. code-block:: bash

    rm -rf certs

.. _private ca https:

Using a certificate signed by a private CA
------------------------------------------

Configuring Media Gateway
^^^^^^^^^^^^^^^^^^^^^^^^^

To use a certificate signed by a private CA update server and client configurations.

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

where

* ``server.crt`` is a file with the server certificate in PEM format.

* ``server.key`` is a file with the server key in PEM format.

* ``ca.crt`` is a file with the CA certificate in PEM format.

Generating certificates
^^^^^^^^^^^^^^^^^^^^^^^

Generate a private key and a certificate for CA and a private key and certificate signing request for the server

.. code-block:: bash

    mkdir certs

    openssl genpkey -algorithm RSA -out certs/ca.key

    openssl req -new -x509 -days 365 -key certs/ca.key -out certs/ca.crt -subj "/CN=media-gateway-ca"

    openssl genpkey -algorithm RSA -out certs/server.key

    openssl req -new -key certs/server.key -out certs/server.csr -subj "/CN=media-gateway-server"

If the client connects to the server by IP generate a certificate with IP subject alternative name. Otherwise generate a certificate with DNS subject alternative name.

In commands below replace `192.168.0.108` and `media-gateway-server` with your values.

.. code-block:: bash
    :caption: IP SAN

    export HOST_IP="192.168.0.108"

    openssl x509 -req -days 365 -in certs/server.csr -CA certs/ca.crt -CAkey certs/ca.key -CAcreateserial -out certs/server.crt -extfile <(echo "subjectAltName=IP:${HOST_IP}")

.. code-block:: bash
    :caption: DNS SAN

    export MEDIA_GATEWAY_SERVER_DNS="media-gateway-server"

    openssl x509 -req -days 365 -in certs/server.csr -CA certs/ca.crt -CAkey certs/ca.key -CAcreateserial -out certs/server.crt -extfile <(echo "subjectAltName=DNS:${MEDIA_GATEWAY_SERVER_DNS}")

Testing
^^^^^^^

Testing is the same as for a self-signed certificate except the request to the server for checking

.. code-block:: bash

    curl --cacert certs/ca.crt -v https://$HOST_IP:$MEDIA_GATEWAY_PORT/health

Using a certificate signed by a public CA
-----------------------------------------

Configuring Media Gateway
^^^^^^^^^^^^^^^^^^^^^^^^^

To use a certificate signed by a public CA update the server configuration.

.. code-block:: json
    :caption: server

    "tls": {
        "identity": {
            "certificate": "server.crt",
            "key": "server.key"
        }
    }

where

* ``server.crt`` is a file with a sequence of certificates, the first being the leaf certificate, and the remainder forming the chain of certificates up to and including the trusted root certificate.

* ``server.key`` is a file with the server key in PEM format.
