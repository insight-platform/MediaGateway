Secure communication
====================

Media Gateway supports secure communication between the server and client via following features:

* HTTPS protocol
* authentication

  * client certificate authentication (including client revocation lists)
  * HTTP Basic authentication

Features can be used separately or in combinations. The recommended combinations are

* HTTPS protocol + HTTP Basic authentication
* HTTPS protocol + client certificate authentication
* HTTPS protocol + client certificate authentication + HTTP Basic authentication

Guides how to enable security features in Media Gateway:

* :doc:`/cookbook/0_https`
* :doc:`/cookbook/1_certificate_auth`
* :doc:`/cookbook/2_basic_auth`
