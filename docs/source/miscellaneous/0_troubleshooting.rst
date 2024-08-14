Troubleshooting
===============

This guide explains how to fix issues you might encounter when using Media Gateway.

High CPU usage by Media Gateway client
--------------------------------------

Media Gateway client uses one of strategies to wait for data while reading from ZeroMQ socket. If the chosen strategy does not fit, the client checks for new data too often causing high CPU usage. Try to use another strategy or increase a timeout (see :ref:`wait strategy configuration <wait strategy configuration>`).
