.. Media Gateway documentation master file

Media Gateway's documentation
=============================

`Media Gateway <https://github.com/insight-platform/MediaGateway>`_ is an application that provides a functionality to forward `Savant <https://docs.savant-ai.io/>`_ messages from one `ZeroMQ <https://zeromq.org/>`_ instance to another. The media gateway consists of two parts - a server and client. The client reads messages from the source ZeroMQ instance and sends them to the server via HTTP/HTTPS. The server writes received messages to the target ZeroMQ instance.

* **Repository**: https://github.com/insight-platform/MediaGateway
* **License**: Business Source License 1.1

Features
--------
* HTTPS
* basic authentication
* client certificate authentication
* FPS statistics logging (by frame or timestamp period)

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
