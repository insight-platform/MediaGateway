Configuration
=============

.. _server configuration:

Server
------

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
      - A configuration how to write to ZeroMQ socket. See :ref:`sink configuration <sink configuration>`.
      - yes
    * - tls
      - TLS settings. See :ref:`server TLS settings configuration <server tls settings configuration>`.
      - no
    * - auth
      - Authentication settings. See :ref:`server authentication settings configuration <server authentication settings configuration>`.
      - no
    * - statistics
      - Statistics settings. See :ref:`statistics configuration <statistics configuration>`.
      - no

.. _client configuration:

Client
------

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
      - Media Gateway server URL.
      - yes
    * - retry_strategy
      - A strategy how to retry to send a message to Media Gateway server. The default value is an exponential strategy with the initial delay 1 ms, the maximum delay 1 sec and the multiplier 2. See :ref:`retry strategy configuration <retry strategy configuration>`.
      - no
    * - in_stream
      - A configuration how to read from ZeroMQ socket. See :ref:`source configuration <source configuration>`.
      - yes
    * - wait_strategy
      - A strategy how to wait for data from ZeroMQ socket. The default value is 1 ms sleep strategy. See :ref:`wait strategy configuration <wait strategy configuration>`.
      - no
    * - tls
      - TLS settings. See :ref:`client TLS settings configuration <client TLS settings configuration>`.
      - no
    * - auth
      - Authentication settings. See :ref:`client authentication settings configuration <client authentication settings configuration>`.
      - no
    * - statistics
      - Statistics settings. See :ref:`statistics configuration <statistics configuration>`.
      - no

Subconfigurations
-----------------

.. _duration configuration:

Duration
^^^^^^^^

A duration is specified as a composition of a whole number of seconds and a fractional part represented in nanoseconds.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - secs
      - Duration seconds
      - yes
    * - nanos
      - Duration nanoseconds
      - yes

.. _sink configuration:

Sink
^^^^

A configuration how to write to ZeroMQ socket.

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
      - The timeout for sending data. The default value is ok for most cases. See :ref:`duration configuration <duration configuration>`.
      - no
      - ``{"secs": 1, "nanos": 0}``
    * - send_retries
      - The number of retries for sending data. The default value is ok for most cases. For unstable or busy recipients the value might be increased.
      - no
      - ``3``
    * - receive_timeout
      - The timeout for receiving data. Valid only for ``dealer`` and ``req`` socket types. The default value is ok for most cases. See :ref:`duration configuration <duration configuration>`.
      - no
      - ``{"secs": 1, "nanos": 0}``
    * - receive_retries
      - The number of retries for receiving data (crucial for req/rep communication). The default value is ok for most cases. For unstable or busy senders the value might be increased.
      - yes
      - ``3``
    * - send_hwm
      - The high-water mark for sending data. This parameter is used to control backpressure. Consult with ZeroMQ documentation for more details.
      - no
      - ``1000``
    * - receive_hwm
      - The high-water mark for receiving data. This parameter is used to control backpressure. Consult with ZeroMQ documentation for more details. Change only if you are using req/rep communication.
      - no
      - ``1000``
    * - fix_ipc_permissions
      - UNIX file permissions for IPC sockets.
      - no
      -

.. _source configuration:

Source
^^^^^^

A configuration how to read from ZeroMQ socket.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - url
      - The URL in Savant ZMQ format.
      - yes
    * - receive_timeout
      - The timeout for receiving data. The default value is ok for most cases. See :ref:`duration configuration <duration configuration>`.
      - yes
    * - receive_hwm
      - The high-water mark for receiving data. This parameter is used to control backpressure. Consult with ZeroMQ documentation for more details.
      - yes
    * - topic_prefix_spec
      - The topic prefix specification for receiving data. Possible values are ``{"none": null}``, ``{"source_id": "topic"}`` or ``{"prefix": "prefix"}``
      - yes
    * - source_cache_size
      - The size of the whitelist cache used only when prefix-based filtering is in use. This parameter is used to quickly check if the source ID is in the whitelist or must be checked.
      - yes
    * - fix_ipc_permissions
      - UNIX file permissions for IPC sockets.
      - no
    * - inflight_ops
      - The maximum number of read messages for non-blocking mode.
      - yes

.. _retry strategy configuration:

Retry strategy
^^^^^^^^^^^^^^

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - exponential
      - Settings for exponential retry strategy.
      - true

Exponential retry strategy
""""""""""""""""""""""""""

The strategy executes next attempt after the delay which is calculated for each attempt. The delay for the first attempt is the initial delay. The delay for subsequent attempts is calculated as maximum between multiplication of last attempt delay by the specified multiplier and the maximum delay.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - initial_delay
      - The delay with nanosecond precision for the first attempt. See :ref:`duration configuration <duration configuration>`.
      - true
    * - maximum_delay
      - The maximum delay with nanosecond precision. See :ref:`duration configuration <duration configuration>`.
      - true
    * - multiplier
      - A multiplier to calculate the delay for next attempt by multiplying last attempt delay. The minimum value is 2.
      - true

.. _wait strategy configuration:

Wait strategy
^^^^^^^^^^^^^

Yield wait strategy
"""""""""""""""""""

A strategy that pauses execution using `Tokio yield_now <https://docs.rs/tokio/1.39.2/tokio/task/fn.yield_now.html>`__. The strategy does not have configuration parameters and is specified as the string ``yield``.

Sleep wait strategy
"""""""""""""""""""

A strategy that pauses execution using `tokio_timerfd sleep <https://docs.rs/tokio-timerfd/0.2.0/tokio_timerfd/fn.sleep.html>`__ for the specified duration with nanosecond precision.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - sleep
      - A duration with nanosecond precision to sleep. See :ref:`duration configuration <duration configuration>`.
      - true

.. _cache configuration:

Cache
^^^^^

Cache configuration.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - size
      - The maximum number of entries.
      - yes
    * - usage
      - Cache usage settings. See below.
      - no

Cache usage
"""""""""""

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - period
      - A period with millisecond precision to collect data before calculating usage metrics. See :ref:`duration configuration <duration configuration>`.
      - yes
    * - evicted_threshold
      - A number of cache entries allowed for eviction for the period.
      - yes

.. _identity configuration:

Identity
^^^^^^^^

An identity represents a private key and X509 certificate.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - certificate
      - A path to the file with a chain of PEM encoded X509 certificates, the first being the leaf certificate, and the remainder forming the chain of certificates up to and including the trusted root certificate.
      - yes
    * - key
      - A path to the file with a PEM encoded private key.
      - yes

.. _credentials configuration:

Credentials
^^^^^^^^^^^

Credentials represent a username and password.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - username
      - A username.
      - yes
    * - password
      - A password.
      - yes

.. _client tls settings configuration:

Client TLS settings
^^^^^^^^^^^^^^^^^^^

TLS settings used by the client to connect to the server.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - root_certificate
      - A path to the file with a PEM encoded X509 certificate against which to verify the server's TLS certificate. The file might contain a chain of CA certificates, the first being the leaf certificate, and the remainder forming the chain of certificates up to and including the trusted root certificate. For the server with the self-signed certificate the file contains the certificate itself.
      - no
    * - identity
      - An identity to be presented to the server for client certificate authentication. See :ref:`identity configuration <identity configuration>`.
      - no

.. _server tls settings configuration:

Server TLS settings
^^^^^^^^^^^^^^^^^^^

TLS settings for the server.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - identity
      - An identity to be presented to peers. See :ref:`identity configuration <identity configuration>`.
      - yes
    * - peers
      - Settings for peer certificate verification. See below.
      - no

**Peer certificate verification settings**

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - lookup_hash_directory
      - A directory with allowed certificates and CRLs. See `X509_LOOKUP_hash_dir method <https://www.openssl.org/docs/man1.1.1/man3/X509_LOOKUP_hash_dir.html>`_ for more details.
      - yes
    * - crl_enabled
      - ``true`` if CRLs must be checked during certificate verification, ``false`` otherwise.
      - yes

.. _client authentication settings configuration:

Client authentication settings
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Authentication settings for the client.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - basic
      - Credentials used to connect to Media Gateway server. See :ref:`credentials configuration <credentials configuration>`.
      - true

.. _server authentication settings configuration:

Server authentication settings
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Authentication settings for the server.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - basic
      - Settings for HTTP Basic authentication. See below.
      - true

**HTTP Basic authentication settings**

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - etcd
      - etcd configuration. See below.
      - true
    * - cache
      - Settings for authentication cache. See :ref:`cache configuration section <cache configuration>`.
      - true

**etcd**

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - urls
      - A list of etcd server endpoints to connect to.
      - true
    * - tls
      - TLS options to use while connecting to etcd servers. See :ref:`client TLS settings configuration <client tls settings configuration>`.
      - false
    * - credentials
      - Credentials for basic authentication in etcd. See :ref:`credentials configuration <credentials configuration>`.
      - false
    * - path
      - The path of the hierarchically organized directories (as in a standard filesystem) for the stored key/value(-s).
      - true
    * - data_format
      - The format of the data stored in etcd. Possible values are ``json``, ``yaml``.
      - true
    * - connect_timeout
      - A timeout with millisecond precision for each request to etcd. See :ref:`duration configuration <duration configuration>`.
      - true
    * - lease_timeout
      - A timeout with millisecond precision to hold keys if the etcd server does not receive a keepAlive. See :ref:`duration configuration <duration configuration>`.
      - true
    * - cache
      - Settings for user data cache. See :ref:`cache configuration section <cache configuration>`.
      - true

.. _statistics configuration:

Statistics
^^^^^^^^^^

Statistics settings.

.. list-table::
    :header-rows: 1

    * - Field
      - Description
      - Mandatory
    * - frame_period
      - A number of frames to collect before calculating statistics metrics.
      - no*
    * - timestamp_period
      - A period with millisecond precision to collect data before calculating statistics metrics. See :ref:`duration configuration <duration configuration>`.
      - no*

\* exactly one of ``frame_period`` and ``timestamp_period`` must be specified.

Environment variables in configuration files
--------------------------------------------

Environment variables can be used in the configuration file. The syntax is ``${VAR_NAME:-default_value}``. If the environment variable is not set, the default value will be used.

Examples
--------
Examples of configuration files can be found `here <https://github.com/insight-platform/MediaGateway/tree/main/samples/configuration>`_.

