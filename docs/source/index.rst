.. Media Gateway documentation master file

Media Gateway's documentation
=============================

`Media Gateway <https://github.com/insight-platform/MediaGateway>`_ is a service providing a secure bridge (with encryption and authentication) between `Savant <https://docs.savant-ai.io/>`_ edge and cloud components. Media Gateway consists of two elements: a server and client. The client reads messages from the source ZeroMQ socket and sends them to the server via HTTP/HTTPS. The server writes received messages to the target ZeroMQ instance.

* **Repository**: https://github.com/insight-platform/MediaGateway
* **License**: Business Source License 1.1

Features
--------
* HTTPS;
* Etcd-based basic authentication;
* X509 client-certificate authentication.


.. toctree::
   :maxdepth: 1
   :caption: Getting Started

   getting_started/0_configuration
   getting_started/1_deployment

.. toctree::
   :maxdepth: 1
   :caption: Reference

   reference/0_server_api

.. toctree::
   :maxdepth: 1
   :caption: Cookbook

   cookbook/0_tls
   cookbook/1_usage_example
