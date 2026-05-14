<?php
class CompatTry {
    public static function ready() {
        try {
            echo "try";
        } catch (\Throwable $e) {
            echo "catch";
        }
    }
}

echo "AB";
