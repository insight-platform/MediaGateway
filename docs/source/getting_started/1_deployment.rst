Deployment
==========

Both server and client are deployed as Docker containers.

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

To run the server with `the default configuration <https://github.com/insight-platform/MediaGateway/tree/main/samples/configuration/server/default_config.json>`__ and to mount ``/tmp`` directory and publish the port from the default configuration

.. code-block:: bash

    docker run \
        -v /tmp:/tmp \
        -p 8080:8080 \
        ghcr.io/insight-platform/media-gateway-server:latest


To run the server with another configuration (``/home/user/server_config.json``)

.. code-block:: bash

    docker run \
        -v /home/user/server_config.json:/opt/etc/custom_config.json \
        -p HOST_PORT:CONFIG_PORT \
        ghcr.io/insight-platform/media-gateway-server:latest \
        /opt/etc/custom_config.json

Client
------

To run the client with `the default configuration <https://github.com/insight-platform/MediaGateway/tree/main/samples/configuration/client/default_config.json>`__ and to mount ``/tmp`` directory

.. code-block:: bash

    docker run \
        -v /tmp:/tmp \
        -e "GATEWAY_URL=<GATEWAY_URL>" \
        ghcr.io/insight-platform/media-gateway-client:latest

where ``<GATEWAY_URL>`` is the server URL, e.g. ``http://192.168.0.100:8080``

To run the server with another configuration (``/home/user/client_config.json``)

.. code-block:: bash

    docker run \
        -v /home/user/client_config.json:/opt/etc/custom_config.json \
        ghcr.io/insight-platform/media-gateway-client:latest \
        /opt/etc/custom_config.json
