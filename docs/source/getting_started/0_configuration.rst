Configuration
=============

Both server and client applications are configured via JSON files.

Server
------

The server configuration consists of following fields:

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - ip
      - A string representation of an IP address or a host name to bind to. Both IPv4 or IPv6 are supported. If the host name is specified the server is bound to both the IPv4 and IPv6 addresses that result from a DNS lookup.
      - yes
    * - port
      - A port to bind to.
      - yes
    * - out_stream
      - A configuration how to write to the target ZeroMQ.
      - yes
    * - auth
      - Authentication settings.
      - no
    * - ssl
      - HTTPS and client certificate authentication settings.
      - no
    * - statistics
      - Statistics settings.
      - no

out_stream
^^^^^^^^^^

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
      - Default value
    * - url
      - The URL in Savant ZMQ format.
      - yes
      -
    * - send_timeout
      - The timeout for sending data to the egress stream. The default value is .
      - no
      - ``{"secs": 1, "nanos": 0}``
    * - send_retries
      - The number of retries for sending data to the egress stream.
      - no
      - ``3``
    * - receive_timeout
      - The timeout for receiving data from the egress stream. Valid only for dealer and req socket types.
      - no
      - ``{"secs": 1, "nanos": 0}``
    * - receive_retries
      - The number of retries for receiving data from the egress stream (crucial for req/rep communication).
      - yes
      - ``3``
    * - send_hwm
      - The high-water mark for the egress stream. This parameter is used to control backpressure. Please consult with ZeroMQ documentation for more details.
      - no
      - ``1000``
    * - receive_hwm
      - The high-water mark for the egress stream. This parameter is used to control backpressure. Please consult with ZeroMQ documentation for more details.
      - no
      - ``1000``
    * - fix_ipc_permissions
      - If set, Media Gateway will fix the UNIX file permissions for IPC sockets.
      - no
      -

auth
^^^^

The only supported authentication type is ``basic``. Provided user names and passwords are loaded to the memory.

.. code-block:: json

    "auth": {
        "basic": [
            {
                "id": "user",
                "password": "password"
            }
        ]
    }

ssl
^^^
.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - server
      - HTTPS settings.
      - yes
    * - server.certificate
      - A path to a PEM encoded certificate (can be a self-signed certificate).
      - yes
    * - server.certificate_key
      - A path to a private key for the certificate.
      - yes
    * - client
      - Client certificate authentication settings.
      - no
    * - client.certificate_directory
      - A directory with certificates and CRLs to verify client certificates. See `X509_LOOKUP_hash_dir method <https://www.openssl.org/docs/man1.1.1/man3/X509_LOOKUP_hash_dir.html>`_ for more details.
      - yes

statistics
^^^^^^^^^^

At least one of ``frame_period`` and ``timestamp_period`` should be specified.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - frame_period
      - A frame period
      - no
    * - timestamp_period
      - A timestamp period
      - no
    * - history_size
      - A size of a history to be stored
      - yes

Client
------

The client configuration consist of following fields:

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - url
      - An endpoint of the media gateway service to accept messages.
      - yes
    * - in_stream
      - A configuration how to read from the source ZeroMQ.
      - yes
    * - auth
      - Authentication settings.
      - no
    * - ssl
      - HTTPS and client certificate authentication settings.
      - no
    * - statistics
      - Statistics settings.
      - no

in_stream
^^^^^^^^^

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - url
      - The URL for the data ingress in Savant ZMQ format.
      - yes
    * - receive_timeout
      - The timeout for receiving data from the ingress stream.
      - yes
    * - receive_hwm
      - The high-water mark for the ingress stream. This parameter is used to control backpressure. Please consult with ZeroMQ documentation for more details.
      - yes
    * - topic_prefix_spec
      - The topic prefix specification for the ingress stream. Possible values are ``{"none": null}``, ``{"source_id": "topic"}`` or ``{"prefix": "prefix"}``
      - yes
    * - source_cache_size
      - The size of the whitelist cache used only when prefix-based filtering is in use. This parameter is used to quickly check if the source ID is in the whitelist or must be checked.
      - yes
    * - fix_ipc_permissions
      - If set, Media Gateway will fix the UNIX file permissions for IPC sockets.
      - no
    * - inflight_ops
      - The maximum number of read messages for non-blocking mode.
      - yes

auth
^^^^

The only supported authentication type is ``basic``.

.. code-block:: json

    "auth": {
        "basic": {
            "id": "user",
            "password": "password"
        }
    }

ssl
^^^
.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - server
      - HTTPS settings.
      - yes
    * - server.certificate
      - A path to a self-signed PEM encoded server certificate or PEM encoded CA certificate
      - yes
    * - client
      - Client certificate authentication settings.
      - no
    * - client.certificate
      - A path to a chain of PEM encoded X509 certificates, with the leaf certificate first.
      - yes
    * - client.certificate_key
      - A path to a PEM encoded PKCS #8 formatted private key
      - yes

statistics
^^^^^^^^^^

At least one of ``frame_period`` and ``timestamp_period`` should be specified.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - frame_period
      - A frame period
      - no
    * - timestamp_period
      - A timestamp period
      - no
    * - history_size
      - A size of a history to be stored
      - yes


Environment variables in configuration files
--------------------------------------------

You can use environment variables in the configuration file. The syntax is ``${VAR_NAME:-default_value}``. If the environment variable is not set, the default value will be used.

Examples
--------
Examples of configuration files can be found `here <https://github.com/insight-platform/MediaGateway/tree/main/samples/configuration>`_.

