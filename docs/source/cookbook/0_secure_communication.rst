Secure communication
====================

Media Gateway supports TLS for secure communication between the server and the client.

HTTPS configuration
--------------------

Media Gateway supports both self-signed and CA-signed server certificates. Certificates must be provided in PEM format. To enable HTTPS in Media Gateway update both server and client configuration.

The protocol in ``url`` field in the client configuration must be updated to ``https``.

Using a self-signed server certificate
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

To use a self-signed server certificate user must define two configuration parameters:

 * ``server.crt`` is a file with the server certificate in PEM format.
 * ``server.key`` is a file with the server key in PEM format.

.. code-block:: json
    :caption: Server

    "ssl": {
        "server": {
            "certificate": "server.crt",
            "certificate_key": "server.key"
        }
    }

.. code-block:: json
    :caption: Client

    "ssl": {
        "server": {
            "certificate": "server.crt"
        }
    }

Using a CA-signed server certificate
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

To use a CA-signed server certificate user must define two configuration parameters for the server:

 * ``server.crt`` is a file with a sequence of certificates, the first being the leaf certificate, and the remainder forming the chain of certificates up to and including the trusted root certificate.
 * ``server.key`` is a file with the server key in PEM format.

The client configuration must contain the CA certificate:

 * ``ca.crt`` is a file with the CA certificate in PEM format.


.. code-block:: json
    :caption: Server

    "ssl": {
        "server": {
            "certificate": "server.crt",
            "certificate_key": "server.key"
        }
    }

.. code-block:: json
    :caption: Client

    "ssl": {
        "server": {
            "certificate": "ca.crt"
        }
    }

Public CA-signed server certificate
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

To use a public CA-signed server certificate user must define two configuration parameters for the server:

 * ``server.crt`` is a file with a sequence of certificates, the first being the leaf certificate, and the remainder forming the chain of certificates up to and including the trusted root certificate.
 * ``server.key`` is a file with the server key in PEM format.


.. code-block:: json
    :caption: server

    "ssl": {
        "server": {
            "certificate": "server.crt",
            "certificate_key": "server.key"
        }
    }

