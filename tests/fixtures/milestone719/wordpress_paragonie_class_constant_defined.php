<?php
class ParagonIE_Sodium_Compat {
    const LIBRARY_VERSION_MAJOR = 9;
}

$constant = "LIBRARY_VERSION_MAJOR";
echo defined("ParagonIE_Sodium_Compat::$constant") ? "1" : "0";
echo "|", constant("ParagonIE_Sodium_Compat::$constant"), "\n";
