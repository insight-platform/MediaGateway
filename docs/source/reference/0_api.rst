RESTful API
===========

Health
------

Both server and client have a health endpoint.

.. code-block::

    GET /health

If the server/client is healthy an HTTP response with ``200 OK`` status code and the body as below will be returned.

.. code-block:: json

    {
        "status": "healthy"
    }
