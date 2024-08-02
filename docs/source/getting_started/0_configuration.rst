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
    * - tls
      - TLS settings.
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

The only supported authentication type is ``basic``. Provided user names and passwords are loaded from `etcd <https://etcd.io/>`__. The key in `etcd` is a user name. The value in `etcd` is an object in JSON/YAML format with the following schema

.. code-block:: json

    {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "title": "Media Gateway user data schema",
      "type": "object",
      "properties": {
        "password_hash": {
          "description": "Argon2 password hash in PHC string format.",
          "type": "string"
        },
        "allowed_routing_labels": {
          "type": "object",
          "anyOf": [
            {"$ref": "#/$defs/set"},
            {"$ref": "#/$defs/unset"},
            {"$ref": "#/$defs/and"},
            {"$ref": "#/$defs/or"},
            {"$ref": "#/$defs/not"}
          ]
        }
      },
      "required": [ "password_hash" ],
      "$defs": {
        "set": {
          "description": "Set label rule: routing labels must contain a specified label.",
          "type": "string"
        },
        "unset": {
          "description": "Unset label rule: routing labels must not contain a specified label.",
          "type": "string"
        },
        "and" : {
          "description": "And label rule: labels rules combined with and logic.",
          "type": "array",
          "items": {
            "anyOf": [
              {"$ref": "#/$defs/set"},
              {"$ref": "#/$defs/unset"},
              {"$ref": "#/$defs/and"},
              {"$ref": "#/$defs/or"},
              {"$ref": "#/$defs/not"}
            ]
          }
        },
        "or" : {
          "description": "Or label rule: labels rules combined with or logic.",
          "type": "array",
          "items": {
            "anyOf": [
              {"$ref": "#/$defs/set"},
              {"$ref": "#/$defs/unset"},
              {"$ref": "#/$defs/and"},
              {"$ref": "#/$defs/or"},
              {"$ref": "#/$defs/not"}
            ]
          }
        },
        "not" : {
          "description": "Not label rule: a negation of the specified label rule.",
          "type": "object",
          "items": {
            "anyOf": [
              {"$ref": "#/$defs/set"},
              {"$ref": "#/$defs/unset"},
              {"$ref": "#/$defs/and"},
              {"$ref": "#/$defs/or"},
              {"$ref": "#/$defs/not"}
            ]
          }
        }
      }
    }

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - etcd
      - etcd configuration.
      - true
    * - etcd.urls
      - A list of etcd server endpoints to connect to.
      - true
    * - etcd.tls
      - TLS options to use while connecting to etcd servers.
      - false
    * - etcd.tls.root_certificate
      - CA certificate against which to verify the etcd server's TLS certificate.
      - false
    * - etcd.tls.identity
      - The client identity to present to the etcd server.
      - false
    * - etcd.tls.identity.certificate
      - A path to a chain of PEM encoded certificates, with the leaf certificate first.
      - true
    * - etcd.tls.identity.key
      - A path to a PEM encoded private key
      - true
    * - etcd.credentials
      - Credentials for basic authentication in etcd.
      - false
    * - etcd.credentials.username
      - A user name for basic authentication in etcd.
      - true
    * - etcd.credentials.password
      - A password for basic authentication in etcd.
      - true
    * - etcd.path
      - The path of the hierarchically organized directories (as in a standard filesystem) for the stored key/value(-s).
      - true
    * - etcd.data_format
      - The format of the data stored in etcd. Possible values are `json`, `yaml`.
      - true
    * - etcd.connect_timeout
      - A timeout for each request to etcd in the duration format, e.g. ``{"secs": 1, "nanos": 0}``.
      - true
    * - etcd.lease_timeout
      - A timeout to hold keys if the etcd server does not receive a keepAlive, in the duration format, e.g. ``{"secs": 60, "nanos": 0}``
      - true
    * - etcd.cache
      - Settings for user data cache. See :ref:`cache configuration section <cache configuration>`.
      - true
    * - cache
      - Settings for authentication cache. See :ref:`cache configuration section <cache configuration>`.
      - true

.. _cache configuration:

cache
^^^^^

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - size
      - A size of LRU cache for credentials to exclude running encryption functions when the same credentials are passed.
      - true
    * - usage
      - Settings to monitor LRU cache usage and to produce a warning when more than X keys per a period are evicted.
      - false
    * - usage.period
      - A period to track evicted keys in the duration format, e.g. ``{"secs": 1, "nanos": 0}``.
      - true
    * - usage.evicted_threshold
      - The positive integer number of keys allowed for eviction for the period.
      - true

tls
^^^
.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - identity
      - HTTPS settings.
      - yes
    * - identity.certificate
      - A path to a PEM encoded certificate (can be a self-signed certificate).
      - yes
    * - identity.key
      - A path to a private key for the certificate.
      - yes
    * - peers
      - Settings to verify peer certificates.
      - no
    * - peers.lookup_hash_directory
      - A directory with certificates and CRLs to verify client certificates. See `X509_LOOKUP_hash_dir method <https://www.openssl.org/docs/man1.1.1/man3/X509_LOOKUP_hash_dir.html>`_ for more details.
      - yes
    * - peers.crl_enabled
      - ``true`` if CRLs must be checked during client certificate verification, ``false`` otherwise.
      - yes

statistics
^^^^^^^^^^

Exactly one of ``frame_period`` and ``timestamp_period`` must be specified.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - frame_period
      - A frame period
      - no
    * - timestamp_period
      - A timestamp period in the duration format with millisecond precision, e.g. ``{"secs": 1, "nanos": 0}``
      - no

Client
------

The client configuration consist of following fields:

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
    * - url
      - An endpoint of the media gateway service to accept messages.
      - yes
    * - retry_strategy
      - A strategy how to retry to send a message to the media gateway service. The default value is an exponential strategy with the initial delay 1 ms, the maximum delay 1 sec and the multiplier 2.
      - no
    * - in_stream
      - A configuration how to read from the source ZeroMQ.
      - yes
    * - wait_strategy
      - A strategy how to wait for data from the source ZeroMQ. The default value is 1 ms sleep strategy.
      - no
    * - auth
      - Authentication settings.
      - no
    * - tls
      - TLS settings.
      - no
    * - statistics
      - Statistics settings.
      - no

retry_strategy
^^^^^^^^^^^^^^

There is only one retry strategy - exponential. The strategy executes next attempt after the delay which is calculated for each attempt. The delay for the first attempt is the initial delay. The delay for subsequent attempts is calculated as maximum between multiplication of last attempt delay by the specified multiplier and the maximum delay.

Retry strategy is an object with the following schema

.. code-block:: json

    {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "title": "Media Gateway Client retry strategy schema",
      "anyOf": [
        {
          "description": "A strategy that executes next attempt after a delay that starts from the initial delay and increases (multiplying by the specified multiplier) with each attempt up to the maximum.",
          "type": "object",
          "properties": {
            "exponential": {
              "type": "object",
              "properties": {
                "initial_delay": {
                  "description": "The delay for the first attempt.",
                  "$ref": "#/$defs/duration"
                },
                "maximum_delay": {
                  "description": "The maximum delay",
                  "$ref": "#/$defs/duration"
                },
                "multiplier": {
                  "description": "A multiplier to calculate the delay for next attempt by multiplying last attempt delay.",
                  "type": "integer",
                  "minimum": 2
                }
              },
              "required": [
                "initial_delay",
                "maximum_delay",
                "multiplier"
              ]
            }
          },
          "required": [
            "exponential"
          ]
        }
      ],
      "$defs": {
        "duration": {
          "description": "A duration composed of a whole number of seconds and a fractional part represented in nanoseconds.",
          "type": "object",
          "properties": {
            "secs": {
              "description": "Duration seconds.",
              "type": "integer",
              "minimum": 0
            },
            "nanos": {
              "description": "Duration nanoseconds.",
              "type": "integer",
              "minimum": 0,
              "maximum": 999999999
            }
          },
          "required": ["secs", "nanos"]
        }
      }
    }

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

.. _wait strategy:

wait_strategy
^^^^^^^^^^^^^

There are two wait strategies:

* yield

A strategy that pauses execution using `Tokio yield_now <https://docs.rs/tokio/1.39.2/tokio/task/fn.yield_now.html>`__.

* sleep

A strategy that pauses execution using `tokio_timerfd sleep <https://docs.rs/tokio-timerfd/0.2.0/tokio_timerfd/fn.sleep.html>`__ for the specified duration with nanosecond precision.

Wait strategy is an object with the following schema

.. code-block:: json

    {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "title": "Media Gateway Client wait strategy schema",
      "anyOf": [
        {
          "description": "A strategy that pauses execution using https://docs.rs/tokio/1.39.2/tokio/task/fn.yield_now.html.",
          "type": "string",
          "pattern": "^yield$"
        },
        {
          "description": "A strategy that pauses execution using https://docs.rs/tokio-timerfd/0.2.0/tokio_timerfd/fn.sleep.html for the specified duration with nanosecond precision.",
          "type": "object",
          "properties": {
            "sleep": {
              "description": "A duration to sleep composed of a whole number of seconds and a fractional part represented in nanoseconds.",
              "type": "object",
              "properties": {
                "secs": {
                  "description": "Duration seconds.",
                  "type": "integer",
                  "minimum": 0
                },
                "nanos": {
                  "description": "Duration nanoseconds.",
                  "type": "integer",
                  "minimum": 0
                }
              }
            }
          }
        }
      ]
    }

auth
^^^^

The only supported authentication type is ``basic``.

.. code-block:: json

    "auth": {
        "basic": {
            "username": "user",
            "password": "password"
        }
    }

tls
^^^
.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - root_certificate
      - A path to a self-signed PEM encoded server certificate or PEM encoded CA certificate
      - yes
    * - identity
      - Client certificate authentication settings.
      - no
    * - identity.certificate
      - A path to a chain of PEM encoded X509 certificates, with the leaf certificate first.
      - yes
    * - identity.key
      - A path to a PEM encoded PKCS #8 formatted private key
      - yes

statistics
^^^^^^^^^^

Exactly one of ``frame_period`` and ``timestamp_period`` must be specified.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - frame_period
      - A frame period
      - no
    * - timestamp_period
      - A timestamp period in the duration format with millisecond precision, e.g. ``{"secs": 1, "nanos": 0}``
      - no

Environment variables in configuration files
--------------------------------------------

You can use environment variables in the configuration file. The syntax is ``${VAR_NAME:-default_value}``. If the environment variable is not set, the default value will be used.

Examples
--------
Examples of configuration files can be found `here <https://github.com/insight-platform/MediaGateway/tree/main/samples/configuration>`_.

