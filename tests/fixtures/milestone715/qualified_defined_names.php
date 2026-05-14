<?php
echo defined("\\PHP_VERSION_ID") ? "1" : "0";
echo defined("\\Sodium\\CRYPTO_AUTH_BYTES") ? "1" : "0";
echo defined("Sodium\\CRYPTO_AUTH_BYTES") ? "1" : "0";
echo defined("Sodium\\Compat\\CRYPTO_AUTH_BYTES") ? "1" : "0";
