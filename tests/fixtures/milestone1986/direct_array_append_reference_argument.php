<?php
error_reporting(0);

function milestone1986_set_ref(&$value) {
    $value = "direct";
}

$items = array("x" => false);
milestone1986_set_ref($items["x"][]);

echo gettype($items["x"]), "|", $items["x"][0];
