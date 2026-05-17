<?php
$value = "before";
$fn = function () use (&$value) {
    return $value;
};
echo $fn();
