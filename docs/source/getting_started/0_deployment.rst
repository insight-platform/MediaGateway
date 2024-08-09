Deployment
==========

Media Gateway server and client are deployed as Docker containers. Docker images are built for x86_64 and ARM64 architectures and available on GitHub Container Registry.

If you do not want to run Media Gateway in Docker, you can build it from source or contact us for building binaries for your platform.

Common environment variables
----------------------------

.. list-table::
    :header-rows: 1

    * - Variable
      - Description
      - Mandatory
      - Default value
    * - RUST_LOG
      - The log level. Possible values are ``error``, ``warn``, ``info``, ``debug``, ``trace``.
      - no
      - ``info``

Server
------

Docker images:

* `media-gateway-server-x86 <https://github.com/insight-platform/MediaGateway/pkgs/container/media-gateway-server-x86>`__ for x86_64 architecture

* `media-gateway-server-arm64 <https://github.com/insight-platform/MediaGateway/pkgs/container/media-gateway-server-arm64>`__ for ARM64 architecture

The server is configured via a file in JSON format (see :ref:`server configuration <server configuration>`).

Default configuration
^^^^^^^^^^^^^^^^^^^^^

To run the server with `the default configuration <https://github.com/insight-platform/MediaGateway/tree/main/samples/configuration/server/default_config.json>`__, to mount ``/tmp`` directory and publish the port from the default configuration

.. code-block:: bash
    :caption: x86_64

    docker run \
        -v /tmp:/tmp \
        -p 8080:8080 \
        ghcr.io/insight-platform/media-gateway-server-x86:latest

.. code-block:: bash
    :caption: ARM64

    docker run \
        -v /tmp:/tmp \
        -p 8080:8080 \
        ghcr.io/insight-platform/media-gateway-server-arm64:latest

Custom configuration
^^^^^^^^^^^^^^^^^^^^

To run the server with a custom configuration file ``/home/user/server_config.json`` and publish the port specified in it

.. code-block:: bash
    :caption: x86_64

    docker run \
        -v /home/user/server_config.json:/opt/etc/custom_config.json \
        -p <HOST_PORT>:<CONFIG_PORT> \
        ghcr.io/insight-platform/media-gateway-server-x86:latest \
        /opt/etc/custom_config.json

.. code-block:: bash
    :caption: ARM64

    docker run \
        -v /home/user/server_config.json:/opt/etc/custom_config.json \
        -p <HOST_PORT>:<CONFIG_PORT> \
        ghcr.io/insight-platform/media-gateway-server-arm64:latest \
        /opt/etc/custom_config.json

where ``<CONFIG_PORT>`` is the port specified in the configuration file and ``<HOST_PORT>`` is the port on the host machine.

Client
------

Docker images:

* `media-gateway-client-x86 <https://github.com/insight-platform/MediaGateway/pkgs/container/media-gateway-client-x86>`__ for x86_64 architecture

* `media-gateway-client-arm64 <https://github.com/insight-platform/MediaGateway/pkgs/container/media-gateway-client-arm64>`__ for ARM64 architecture

The client is configured via a file in JSON format (see :ref:`client configuration <client configuration>`).

Default configuration
^^^^^^^^^^^^^^^^^^^^^

To run the client with `the default configuration <https://github.com/insight-platform/MediaGateway/tree/main/samples/configuration/client/default_config.json>`__, to mount ``/tmp`` directory and publish the port from the default configuration

.. code-block:: bash
    :caption: x86_64

    docker run \
        -v /tmp:/tmp \
        -p 8081:8081 \
        -e "GATEWAY_URL=<GATEWAY_URL>" \
        ghcr.io/insight-platform/media-gateway-client-x86:latest

.. code-block:: bash
    :caption: ARM64

    docker run \
        -v /tmp:/tmp \
        -p 8081:8081 \
        -e "GATEWAY_URL=<GATEWAY_URL>" \
        ghcr.io/insight-platform/media-gateway-client-arm64:latest

where ``<GATEWAY_URL>`` is Media Gateway server URL, e.g. ``http://192.168.0.100:8080``

Custom configuration
^^^^^^^^^^^^^^^^^^^^

To run the client with a custom configuration file ``/home/user/client_config.json`` and publish the port specified in it

.. code-block:: bash
    :caption: x86_64

    docker run \
        -v /home/user/client_config.json:/opt/etc/custom_config.json \
        -p <HOST_PORT>:<CONFIG_PORT> \
        ghcr.io/insight-platform/media-gateway-client-x86:latest \
        /opt/etc/custom_config.json

.. code-block:: bash
    :caption: arm64

    docker run \
        -v /home/user/client_config.json:/opt/etc/custom_config.json \
        -p <HOST_PORT>:<CONFIG_PORT> \
        ghcr.io/insight-platform/media-gateway-client-arm64:latest \
        /opt/etc/custom_config.json

where ``<CONFIG_PORT>`` is the port specified in the configuration file and ``<HOST_PORT>`` is the port on the host machine.
