<?php
class Compat {
    public static function add($val, $addv) {
        if (strlen($val) !== strlen($addv)) {
            throw new SodiumException("values must have the same length");
        }
        return $val . $addv;
    }
}

echo Compat::add("A", "B");
