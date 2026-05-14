<?php
namespace Sodium;

class ParagonIE_Sodium_Compat {
    public const CRYPTO_AUTH_BYTES = 32;
}

const CRYPTO_AUTH_BYTES = ParagonIE_Sodium_Compat::CRYPTO_AUTH_BYTES;
const CRYPTO_SECRETBOX_KEYBYTES = 32;

echo defined("\\Sodium\\CRYPTO_AUTH_BYTES") ? "1" : "0";
echo "|", constant("\\Sodium\\CRYPTO_AUTH_BYTES");
echo "|", defined("Sodium\\CRYPTO_SECRETBOX_KEYBYTES") ? "1" : "0";
echo "|", defined("CRYPTO_AUTH_BYTES") ? "1" : "0";
