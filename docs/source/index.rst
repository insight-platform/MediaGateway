.. Media Gateway documentation master file

Media Gateway's documentation
=============================

Media Gateway is a service that provides a secure bridge (with encryption and authentication) between `Savant <https://docs.savant-ai.io/>`_ edge and cloud components by forwarding messages from one `ZeroMQ <https://zeromq.org/>`_ socket to another. The media gateway consists of two parts - a server and client. The client reads messages from the source ZeroMQ socket and sends them to the server via HTTP/HTTPS. The server writes received messages to the target ZeroMQ socket.

.. image:: _static/media-gateway.png
    :width: 1281
    :align: center

* **Repository**: https://github.com/insight-platform/MediaGateway
* **License**: Business Source License 1.1

Features
--------
* HTTPS
* HTTP Basic authentication with `etcd <https://etcd.io/>`__ as a credentials storage
* X509 client certificate authentication

.. toctree::
   :maxdepth: 1
   :caption: Getting Started

   getting_started/0_deployment
   getting_started/1_secure_communication

.. toctree::
   :maxdepth: 1
   :caption: Reference

   reference/0_configuration
   reference/1_api

.. _cookbook:

.. toctree::
   :maxdepth: 1
   :caption: Cookbook

   cookbook/0_https
   cookbook/1_certificate_auth
   cookbook/2_basic_auth
   cookbook/3_usage_example

.. _miscellaneous:

.. toctree::
   :maxdepth: 1
   :caption: Miscellaneous

   miscellaneous/0_troubleshooting
   miscellaneous/1_benchmarking
   miscellaneous/2_caching
