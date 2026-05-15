<?php
$constant = "CRYPTO_AUTH_BYTES";
echo defined("SODIUM_{$constant}") ? "1" : "0";
echo "|after\n";
