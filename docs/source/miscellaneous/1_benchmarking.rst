Benchmarking
============

Media Gateway can collect statistics and report it to logs. Statistics includes FPS (frames per second) metric which can be used for benchmarking. FPS metric can be calculated by a timestamp period or frame period.

Statistics by timestamp period
------------------------------

Statics calculation by a timestamp period means that data is collected during the timeout and by the end of the timeout metrics are calculated from the collected data.

.. code-block::
    :caption: statistics logs

    [2024-08-02T08:38:52Z INFO  savant_core::pipeline::stats] Time-based FPS counter triggered: FPS = 2536.46, OPS = 0.00, frame_delta = 2539, time_delta = 1.001 sec , period=[1722587931826, 1722587932827] ms
    [2024-08-02T08:38:53Z INFO  savant_core::pipeline::stats] Time-based FPS counter triggered: FPS = 2501.00, OPS = 0.00, frame_delta = 2501, time_delta = 1 sec , period=[1722587932827, 1722587933827] ms
    [2024-08-02T08:38:54Z INFO  savant_core::pipeline::stats] Time-based FPS counter triggered: FPS = 2530.00, OPS = 0.00, frame_delta = 2530, time_delta = 1 sec , period=[1722587933827, 1722587934827] ms

Statistics by frame period
--------------------------

Statics calculation by a frame period means that data is collected until the number of collected frames reaches the specified number and then metrics are calculated from the collected data.

.. code-block::
    :caption: statistics logs

    [2024-08-02T08:37:56Z INFO  savant_core::pipeline::stats] Frame-based FPS counter triggered: FPS = 2386.63, OPS = 0.00, frame_delta = 1000, time_delta = 0.419 sec, period=[1722587875889, 1722587876308] ms
    [2024-08-02T08:37:56Z INFO  savant_core::pipeline::stats] Frame-based FPS counter triggered: FPS = 2427.18, OPS = 0.00, frame_delta = 1000, time_delta = 0.412 sec, period=[1722587876308, 1722587876720] ms
    [2024-08-02T08:37:57Z INFO  savant_core::pipeline::stats] Frame-based FPS counter triggered: FPS = 2487.56, OPS = 0.00, frame_delta = 1000, time_delta = 0.402 sec, period=[1722587876720, 1722587877122] ms

Configuring Media Gateway
-------------------------

Both server and client can be configured to enable statistics. See :doc:`/reference/0_configuration`.

.. code-block:: json
    :caption: by a timestamp period

      "statistics": {
        "timestamp_period": {
          "secs": 1,
          "nanos": 0
        }
      }

.. code-block:: json
    :caption: by a frame period

      "statistics": {
        "frame_period": 1000
      }
