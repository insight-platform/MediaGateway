Caching
=======

Media Gateway server uses several caches to speed up processing.

User data cache
---------------

User data such as usernames, passwords and allowed routing labels are required for HTTP Basic authentication and authorization and stored in `etcd` (see :doc:`/cookbook/2_basic_auth`). User data is cached and automatically reloaded from `etcd` when its checksum is changed.

Authentication caching structures
---------------------------------

Check result cache
^^^^^^^^^^^^^^^^^^

A separate cache is used to decrease cryptographic costs for HTTP Basic authentication (see :doc:`/cookbook/2_basic_auth`). It holds results of authentication checks which are used for subsequent requests if provided credentials are the same and the user's password is not changed.

Failed attempt cache
^^^^^^^^^^^^^^^^^^^^

If a quarantine feature is enabled a separate cache to track failed attempts to authenticate by users is used. It uses the same configuration as authentication check result cache.

Quarantine
^^^^^^^^^^

If a quarantine feature is enabled a separate structure is used to hold names of users that are in quarantine for the specified duration. It uses the same configuration as authentication check result cache and an additional parameter - the duration.

Cache configuration
-------------------

Caching structures use LRU eviction policy. The maximum number of entries the caching structure may contain is specified in the configuration. The caching structure might be inefficient if its size is not suitable. To detect such cases usage tracking is supported. Usage statistics includes evicted entries per period metric. If the metric value exceeds the threshold a warning is reported to logs.

.. code-block::
    :caption: logs for the exceeded evicted entries threshold in user data cache

    [2024-08-05T04:40:20Z WARN  media_gateway_server::server::service::cache] Evicted entities threshold is exceeded for user: 7 per 60.001 seconds

.. code-block::
    :caption: logs for the exceeded evicted entries threshold in authentication check result cache

    [2024-08-05T04:40:20Z WARN  media_gateway_server::server::service::cache] Evicted entities threshold is exceeded for auth check result: 14 per 60.001 seconds

.. code-block::
    :caption: logs for the exceeded evicted entries threshold in authentication failed attempt cache

    [2024-08-05T04:40:20Z WARN  media_gateway_server::server::service::cache] Evicted entities threshold is exceeded for auth failed attempt: 14 per 60.001 seconds

.. code-block::
    :caption: logs for the exceeded evicted entries threshold in authentication quarantine

    [2024-08-05T04:40:20Z WARN  media_gateway_server::server::service::cache] Evicted entities threshold is exceeded for auth quarantine: 14 per 60.001 seconds

The period and the threshold are specified in the configuration (see :ref:`cache configuration <cache configuration>`).

.. code-block:: json
    :caption: cache configuration without cache usage

    {
      "size": 10,
    }

.. code-block:: json
    :caption: cache configuration with cache usage

    {
      "size": 10,
      "usage": {
        "period": {
          "secs": 60,
          "nanos": 0
        },
        "evicted_threshold": 10
      }
    }
