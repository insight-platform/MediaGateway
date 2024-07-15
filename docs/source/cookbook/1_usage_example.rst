Usage example
=============

This example shows how to ingest `the video file <https://eu-central-1.linodeobjects.com/savant-data/demo/shuffle_dance.mp4>`__ to Media Gateway deployed on a local machine using `video loop adapter <https://docs.savant-ai.io/develop/savant_101/10_adapters.html#video-loop-source-adapter>`__ and re-stream it to `AO-RTSP <https://docs.savant-ai.io/develop/savant_101/10_adapters.html#always-on-rtsp-sink-adapter>`__ with REST API.

Prerequisites
-------------

`Docker <https://www.docker.com/>`__ and `Docker Compose <https://docs.docker.com/compose/>`__ are installed and `NVIDIA GPU <https://docs.docker.com/config/containers/resource_constraints/#gpu>`__ is configured.

Launch
------

Download :download:`example files </_download/e2e_usage_video_loop_ao_rtsp.tar.gz>` and then

.. code-block:: bash

    mkdir e2e_usage_video_loop_ao_rtsp & tar -xzf e2e_usage_video_loop_ao_rtsp.tar.gz -C e2e_usage_video_loop_ao_rtsp

    cd e2e_usage_video_loop_ao_rtsp

    wget https://eu-central-1.linodeobjects.com/savant-data/demo/shuffle_dance.mp4

.. code-block:: bash
    :caption: x86_64

    docker compose -f docker-compose-x86.yaml up -d

.. code-block:: bash
    :caption: ARM64

    docker compose -f docker-compose-arm64.yaml up -d

Open the following URL in your browser to view the video: http://127.0.0.1:888/stream/e2e_usage_video_loop_ao_rtsp/

or with FFplay:

.. code-block:: bash

    ffplay rtsp://127.0.0.1:554/stream/e2e_usage_video_loop_ao_rtsp

Termination
-----------

.. code-block:: bash
    :caption: x86_64

    docker compose -f docker-compose-x86.yaml down

.. code-block:: bash
    :caption: ARM64

    docker compose -f docker-compose-arm64.yaml down
