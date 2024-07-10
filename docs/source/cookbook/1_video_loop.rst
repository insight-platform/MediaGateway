Video loop example
==================

This example shows how to ingest video file to Media Gateway deployed on a local machine using `video loop adapter <https://docs.savant-ai.io/v0.4.0/savant_101/10_adapters.html#video-loop-source-adapter>`__.

Preparation
-----------

.. code-block:: bash

    mkdir -p /tmp/media-gateway-video-loop/config /tmp/media-gateway-video-loop/socket /tmp/media-gateway-video-loop/download

Server
------

To run Media Gateway server with the default configuration

.. code-block:: bash

    echo '{
      "ip": "0.0.0.0",
      "port": 8080,
      "out_stream": {
        "url": "pub+bind:ipc:///tmp/socket/server",
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
      },
      "statistics": {
        "timestamp_period": 1000,
        "history_size": 1000
      }
    }' > /tmp/media-gateway-video-loop/config/server_config.json

    docker run --rm --name media-gateway-video-loop-server \
        -v /tmp/media-gateway-video-loop/socket:/tmp/socket \
        -v /tmp/media-gateway-video-loop/config/server_config.json:/opt/etc/custom_config.json:ro \
        -p 8080:8080 \
        ghcr.io/insight-platform/media-gateway-server:latest \
        /opt/etc/custom_config.json

Client
------

To run Media Gateway client with the default configuration

.. code-block:: bash

    echo '{
      "url": "http://localhost:8080",
      "in_stream": {
        "url": "rep+bind:ipc:///tmp/socket/client",
        "receive_timeout": {
          "secs": 10,
          "nanos": 0
        },
        "receive_hwm": 1000,
        "topic_prefix_spec": {
          "none": null
        },
        "source_cache_size": 1000,
        "inflight_ops": 100
      },
      "statistics": {
        "timestamp_period": 1000,
        "history_size": 1000
      }
    }
    ' > /tmp/media-gateway-video-loop/config/client_config.json

    docker run --rm --name media-gateway-video-loop-client \
        --network host \
        -v /tmp/media-gateway-video-loop/socket:/tmp/socket \
        -v /tmp/media-gateway-video-loop/config/client_config.json:/opt/etc/custom_config.json:ro \
        ghcr.io/insight-platform/media-gateway-client:latest \
        /opt/etc/custom_config.json

Video loop
----------

To run video loop adapter with the video file ``video.mp4``

.. code-block:: bash

    docker run --rm -it --name media-gateway-video-loop-source \
        --entrypoint /opt/savant/adapters/gst/sources/video_loop.sh \
        -e SYNC_OUTPUT=True \
        -e ZMQ_ENDPOINT=req+connect:ipc:///tmp/socket/client \
        -e SOURCE_ID=media-gateway-video-loop \
        -e LOCATION=/tmp/video.mp4 \
        -e DOWNLOAD_PATH=/tmp/download \
        -v video.mp4:/tmp/video.mp4:ro \
        -v /tmp/media-gateway-video-loop/socket:/tmp/socket \
        -v /tmp/media-gateway-video-loop/download:/tmp/download \
        ghcr.io/insight-platform/savant-adapters-gstreamer:latest
