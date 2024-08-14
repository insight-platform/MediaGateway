HTTP Basic authentication
=========================

Media Gateway can control access to the server via a username/password authentication (HTTP Basic Authentication). HTTP Basic authentication should be used with HTTPS (see :doc:`0_https`) to provide confidentiality. The only endpoint that is available to anyone is :ref:`a health endpoint <health endpoint>`. Usernames and passwords are taken from `etcd <https://etcd.io/>`__. An optional quarantine feature is available. A user is quarantined for a period if the amount of failed attempts to authenticate reaches the maximum. An error response without password checks is returned if the user is quarantined which reduces risks of attacks (e.g. password hacking, DoS).

`Savant messages <https://github.com/insight-platform/savant-rs/blob/main/savant_core/src/message.rs>`__ contain routing labels. Media Gateway server can be configured to accept messages from a user only if routing labels are allowed. Allowed labels in the form of `a label filter rule <https://github.com/insight-platform/savant-rs/blob/main/savant_core/src/message/label_filter.rs>`__ are taken from `etcd`.

Prerequisites
-------------

* Docker
* Docker Compose
* openssl
* curl

Configuring Media Gateway
-------------------------

To use HTTP Basic authentication update server and client configurations. Examples below show the full possible configuration. See :doc:`/reference/0_configuration` and :doc:`/miscellaneous/2_caching` for more details.

.. code-block:: json
    :caption: server

    "auth": {
        "basic": {
            "etcd": {
                "urls": [
                    "https://etcd:2379"
                ],
                "tls": {
                    "root_certificate": "etcd-ca.crt",
                    "identity": {
                        "certificate": "etcd-client.crt",
                        "key": "etcd-client.key"
                    }
                },
                "credentials": {
                    "username": "etcd-user",
                    "password": "etcd-password"
                },
                "path": "/users",
                "data_format": "json",
                "lease_timeout": {
                    "secs": 60,
                    "nanos": 0
                },
                "connect_timeout": {
                    "secs": 30,
                    "nanos": 0
                },
                "cache": {
                    "size": 10,
                    "usage": {
                        "period": {
                            "secs": 60,
                            "nanos": 0
                        },
                        "evicted_threshold": 10
                    }
                }
            },
            "cache": {
                "size": 10,
                "usage": {
                    "period": {
                        "secs": 60,
                        "nanos": 0
                    },
                    "evicted_threshold": 10
                }
            },
            "quarantine": {
                "failed_attempt_limit": 3,
                "period": {
                    "secs": 60,
                    "nanos": 0
                }
            }
        }
    }

.. code-block:: json
    :caption: client

    "auth": {
        "basic": {
            "username": "user",
            "password": "password"
        }
    }

where

* ``etcd-ca.crt`` is a file with the CA certificate in PEM format.

* ``etcd-client.crt`` is a file with the client in PEM format.

* ``etcd-client.key`` is a file with the client key in PEM format.


Running etcd with TLS authentication
------------------------------------

In order to expose the etcd API to clients outside of the Docker host use the host IP address when configuring etcd. In the command below replace `192.168.0.108` with your value.

.. code-block:: bash

    export HOST_IP="192.168.0.108"

Generating certificates
^^^^^^^^^^^^^^^^^^^^^^^

Generate certificates signed by a private CA

.. code-block:: bash

    mkdir certs

    # Generate CA private key
    openssl genpkey -algorithm RSA -out certs/ca.key

    # Generate CA self-signed certificate
    openssl req -new -x509 -days 365 -key certs/ca.key -out certs/ca.crt -subj "/CN=etcd-ca"

    # Generate server private key
    openssl genpkey -algorithm RSA -out certs/server.key

    # Generate server CSR
    openssl req -new -key certs/server.key -out certs/server.csr -subj "/CN=etcd-server"

    # Generate server certificate signed by the CA with IP address subject alternative name
    openssl x509 -req -days 365 -in certs/server.csr -CA certs/ca.crt -CAkey certs/ca.key -CAcreateserial -out certs/server.crt -extfile <(echo "subjectAltName=IP:${HOST_IP}")

    # Generate client private key
    openssl genpkey -algorithm RSA -out certs/client.key

    # Generate client CSR
    openssl req -new -key certs/client.key -out certs/client.csr -subj "/CN=etcd-client"

    # Generate client certificate signed by the CA
    openssl x509 -req -days 365 -in certs/client.csr -CA certs/ca.crt -CAkey certs/ca.key -CAcreateserial -out certs/client.crt

Launching etcd
^^^^^^^^^^^^^^

Environment variables below declare the docker image and the port on the host for etcd.

.. code-block:: bash

    ETCD_IMAGE="bitnami/etcd:3.5"
    ETCD_PORT=42379

Launch etcd

.. code-block:: bash

    docker run -d \
        -p $ETCD_PORT:2379 \
        -e ETCD_TRUSTED_CA_FILE=/etc/certs/ca.crt \
        -e ETCD_CERT_FILE=/etc/certs/server.crt \
        -e ETCD_KEY_FILE=/etc/certs/server.key \
        -e ETCD_LISTEN_CLIENT_URLS=https://0.0.0.0:2379 \
        -e ETCD_ADVERTISE_CLIENT_URLS=https://0.0.0.0:$ETCD_PORT \
        -e ETCD_CLIENT_CERT_AUTH=true \
        -e ALLOW_NONE_AUTHENTICATION=yes \
        -v $(pwd)/certs:/etc/certs \
        --name etcd \
        $ETCD_IMAGE

Creating a user
---------------

Creating a password hash
^^^^^^^^^^^^^^^^^^^^^^^^

To generate an Argon2 password hash use any utility.

Valid Argon2 hashes for passwords used in this guide

========= ==================================================================================================
password     Argon2 password hash
========= ==================================================================================================
password  $argon2i$v=19$m=12,t=3,p=1$RzNHVVBjQXo4WUNBUUZYSnlOaGc$9Jmizcl1dv6maVzyIiuMV1OB1P9l6PKLbdmNjJDIgaU
password1 $argon2i$v=19$m=12,t=3,p=1$YXkzZmx1eTFwVW5hZ0R2S1dXazA$VxVMw2Omh1CeVqry8Cay+4OZ69OGvn4fma2M5rURZhI
password2 $argon2i$v=19$m=12,t=3,p=1$c0ZYQ1d3VWxabmx0ZUVmWDNIeVk$qHLr2T3xvedA5zZfTZhbNt3sXB9pa/xlFQ9dVmZG8DQ
========= ==================================================================================================

Preparing user data
^^^^^^^^^^^^^^^^^^^

User data in `etcd` are stored as an object in JSON/YAML format with the following schema

.. code-block:: json

    {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "title": "Media Gateway user data schema",
      "type": "object",
      "properties": {
        "password_hash": {
          "description": "Argon2 password hash in PHC string format.",
          "type": "string"
        },
        "allowed_routing_labels": {
          "type": "object",
          "anyOf": [
            {"$ref": "#/$defs/set"},
            {"$ref": "#/$defs/unset"},
            {"$ref": "#/$defs/and"},
            {"$ref": "#/$defs/or"},
            {"$ref": "#/$defs/not"}
          ]
        }
      },
      "required": [ "password_hash" ],
      "$defs": {
        "set": {
          "description": "Set label rule: routing labels must contain a specified label.",
          "type": "string"
        },
        "unset": {
          "description": "Unset label rule: routing labels must not contain a specified label.",
          "type": "string"
        },
        "and" : {
          "description": "And label rule: labels rules combined with and logic.",
          "type": "array",
          "items": {
            "anyOf": [
              {"$ref": "#/$defs/set"},
              {"$ref": "#/$defs/unset"},
              {"$ref": "#/$defs/and"},
              {"$ref": "#/$defs/or"},
              {"$ref": "#/$defs/not"}
            ]
          }
        },
        "or" : {
          "description": "Or label rule: labels rules combined with or logic.",
          "type": "array",
          "items": {
            "anyOf": [
              {"$ref": "#/$defs/set"},
              {"$ref": "#/$defs/unset"},
              {"$ref": "#/$defs/and"},
              {"$ref": "#/$defs/or"},
              {"$ref": "#/$defs/not"}
            ]
          }
        },
        "not" : {
          "description": "Not label rule: a negation of the specified label rule.",
          "type": "object",
          "items": {
            "anyOf": [
              {"$ref": "#/$defs/set"},
              {"$ref": "#/$defs/unset"},
              {"$ref": "#/$defs/and"},
              {"$ref": "#/$defs/or"},
              {"$ref": "#/$defs/not"}
            ]
          }
        }
      }
    }

**Examples**

.. code-block:: json
    :caption: user data without allowed routing labels in JSON

    {
      "password_hash": "$argon2i$v=19$m=12,t=3,p=1$YXkzZmx1eTFwVW5hZ0R2S1dXazA$VxVMw2Omh1CeVqry8Cay+4OZ69OGvn4fma2M5rURZhI"
    }

.. code-block:: json
    :caption: user data with a set label rule in JSON

    {
      "password_hash": "$argon2i$v=19$m=12,t=3,p=1$YXkzZmx1eTFwVW5hZ0R2S1dXazA$VxVMw2Omh1CeVqry8Cay+4OZ69OGvn4fma2M5rURZhI",
      "allowed_routing_labels": {
        "set": "label"
      }
    }

Saving user data
^^^^^^^^^^^^^^^^

Save data with a password `password1` for a user with the name `user1` in etcd under the path `/users`

.. code-block:: bash

    docker run -it --rm \
        -v $(pwd)/certs:/etc/certs \
        $ETCD_IMAGE \
        etcdctl \
        --cacert /etc/certs/ca.crt \
        --cert /etc/certs/client.crt \
        --key /etc/certs/client.key \
        --endpoints https://$HOST_IP:$ETCD_PORT \
        put \
        /users/user1 \
        '{"password_hash": "$argon2i$v=19$m=12,t=3,p=1$YXkzZmx1eTFwVW5hZ0R2S1dXazA$VxVMw2Omh1CeVqry8Cay+4OZ69OGvn4fma2M5rURZhI"}'

Testing
-------

Server
^^^^^^

To test the server only prepare a configuration file. The configuration below does not contain TLS settings for simplicity. For production HTTP Basic authentication should be used with HTTPS (see :doc:`0_https`).

.. code-block:: bash

    cat << EOF > media-gateway-server.json
    {
        "ip": "0.0.0.0",
        "port": 8080,
        "auth": {
            "basic": {
                "etcd": {
                    "urls": [
                        "https://$HOST_IP:$ETCD_PORT"
                    ],
                    "tls": {
                        "root_certificate": "/etc/certs/ca.crt",
                        "identity": {
                            "certificate": "/etc/certs/client.crt",
                            "key": "/etc/certs/client.key"
                        }
                    },
                    "path": "/users",
                    "data_format": "json",
                    "lease_timeout": {
                        "secs": 60,
                        "nanos": 0
                    },
                    "connect_timeout": {
                        "secs": 30,
                        "nanos": 0
                    },
                    "cache": {
                        "size": 10,
                        "usage": {
                            "period": {
                                "secs": 60,
                                "nanos": 0
                            },
                            "evicted_threshold": 10
                        }
                    }
                },
                "cache": {
                    "size": 10,
                    "usage": {
                        "period": {
                            "secs": 60,
                            "nanos": 0
                        },
                        "evicted_threshold": 10
                    }
                },
                "quarantine": {
                    "failed_attempt_limit": 3,
                    "period": {
                        "secs": 60,
                        "nanos": 0
                    }
                }
            }
        },
        "out_stream": {
            "url": "pub+bind:ipc:///tmp/server",
            "send_timeout": {
                "secs": 1,
                "nanos": 0
            },
            "send_retries": 3,
            "receive_timeout": {
                "secs": 1,
                "nanos": 0
            },
            "receive_retries": 3,
            "send_hwm": 1000,
            "receive_hwm": 1000,
            "fix_ipc_permissions": 511
        }
    }
    EOF

Launch the server (change the value of `MEDIA_GATEWAY_PORT` in the command below if required)

.. code-block:: bash
    :caption: x86_64

    export MEDIA_GATEWAY_PORT=8080

    docker run -d \
        -v $(pwd)/media-gateway-server.json:/opt/etc/custom_config.json \
        -v $(pwd)/certs:/etc/certs \
        -p $MEDIA_GATEWAY_PORT:8080 \
        --name media-gateway-server \
        ghcr.io/insight-platform/media-gateway-server-x86:latest \
        /opt/etc/custom_config.json

.. code-block:: bash
    :caption: ARM64

    export MEDIA_GATEWAY_PORT=8080

    docker run -d \
        -v $(pwd)/media-gateway-server.json:/opt/etc/custom_config.json \
        -v $(pwd)/certs:/etc/certs \
        -p $MEDIA_GATEWAY_PORT:8080 \
        --name media-gateway-server \
        ghcr.io/insight-platform/media-gateway-server-arm64:latest \
        /opt/etc/custom_config.json

For simplicity an invalid request is used for testing. Send a request with a valid user name and password.

.. code-block:: bash

    curl -v -u user1:password1 http://$HOST_IP:$MEDIA_GATEWAY_PORT/ -X POST

HTTP response with ``400 Bad Request`` status code should be returned. It means that authentication is successful.

Send a request with an invalid user name and password.

.. code-block:: bash

    curl -v -u user1:password http://$HOST_IP:$MEDIA_GATEWAY_PORT/ -X POST

HTTP response with ``401 Unauthorized`` status code should be returned. It means that authentication fails.

Send the last request two more times. Each time HTTP response with ``401 Unauthorized`` status code should be returned. After that send the request with the valid password. HTTP response with ``401 Unauthorized`` status code should be returned during 1 minute, after 1 minute - HTTP response with ``400 Bad Request`` status code.

Add a new user `user2` with a password `password2` and send a request using it to test that new users are loaded.

.. code-block:: bash

    docker run -it --rm \
        -v $(pwd)/certs:/etc/certs \
        $ETCD_IMAGE \
        etcdctl \
        --cacert /etc/certs/ca.crt \
        --cert /etc/certs/client.crt \
        --key /etc/certs/client.key  \
        --endpoints https://$HOST_IP:$ETCD_PORT \
        put \
        /users/user2 \
        '{"password_hash": "$argon2i$v=19$m=12,t=3,p=1$c0ZYQ1d3VWxabmx0ZUVmWDNIeVk$qHLr2T3xvedA5zZfTZhbNt3sXB9pa/xlFQ9dVmZG8DQ"}'

    curl -v -u user2:password2 http://$HOST_IP:$MEDIA_GATEWAY_PORT/ -X POST

Change the password for the user `user2` to `password` and send a request using the old and new password to test that users are updated.

.. code-block:: bash

    docker run -it --rm \
        -v $(pwd)/certs:/etc/certs \
        $ETCD_IMAGE \
        etcdctl \
        --cacert /etc/certs/ca.crt \
        --cert /etc/certs/client.crt \
        --key /etc/certs/client.key \
        --endpoints https://$HOST_IP:$ETCD_PORT \
        put \
        /users/user2 \
        '{"password_hash": "$argon2i$v=19$m=12,t=3,p=1$RzNHVVBjQXo4WUNBUUZYSnlOaGc$9Jmizcl1dv6maVzyIiuMV1OB1P9l6PKLbdmNjJDIgaU"}'

    curl -v -u user2:password2 http://$HOST_IP:$MEDIA_GATEWAY_PORT/ -X POST

    curl -v -u user2:password http://$HOST_IP:$MEDIA_GATEWAY_PORT/ -X POST

Clean up after testing

.. code-block:: bash

    docker stop media-gateway-server etcd

    docker rm media-gateway-server etcd

    rm -rf certs media-gateway-server.json

e2e
^^^

To test both server and client based on :doc:`3_usage_example`

* update ``server_config.json`` and ``client_config.json`` in the downloaded archive as described above
* add volumes for ``media-gateway-server`` (key and certificate files) in ``docker-compose-x86.yaml`` and ``docker-compose-arm64.yaml``  in the downloaded archive

Clean up after testing

.. code-block:: bash

    docker stop etcd

    docker rm etcd

    rm -rf certs
