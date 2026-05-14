<?php
class CompatCast {
    public static function compare($left, $right) {
        return (int) sodium_compare($left, $right);
    }
}

echo "after";
