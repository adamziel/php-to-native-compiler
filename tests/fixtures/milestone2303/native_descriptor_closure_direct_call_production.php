<?php
$prefix = "Ada";
$call = function ($value, $suffix = "!") use ($prefix) {
    return $prefix . ":" . $value . $suffix;
};

echo $call("Lovelace"), "\n";
