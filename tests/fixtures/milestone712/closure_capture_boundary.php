<?php
$value = 1;
$fn = function () use ($value) {
    return $value;
};
echo "unreached";
