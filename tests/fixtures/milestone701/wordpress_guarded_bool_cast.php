<?php
class CompatBoolCast {
    public static function supportsGcm() {
        return (bool) in_array('aes-256-gcm', openssl_get_cipher_methods());
    }
}

echo "after";
