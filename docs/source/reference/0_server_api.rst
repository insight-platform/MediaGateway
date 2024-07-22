RESTful API
===========

Health Check
-------------

The server and client provide a health check endpoint available for the end-user.

.. code-block::

    GET /health

If the server is healthy an HTTP response with ``200 OK`` status code and the body as below will be returned.

.. code-block:: json

    {
        "status": "healthy"
    }
