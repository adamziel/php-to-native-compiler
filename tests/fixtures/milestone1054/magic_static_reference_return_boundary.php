<?php
class Box {
    public static function &__callStatic($method, $args) {
        return $args[0];
    }
}

$value = 1;
$alias =& Box::missing($value);
