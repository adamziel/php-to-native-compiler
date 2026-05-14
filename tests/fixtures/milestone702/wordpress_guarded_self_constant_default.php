<?php
class SodiumCompatDefaultProbe {
    const CRYPTO_GENERICHASH_BYTES = 32;

    public static function crypto_generichash(
        $message,
        $key = '',
        $length = self::CRYPTO_GENERICHASH_BYTES
    ) {
        return $length;
    }
}

echo "after";
