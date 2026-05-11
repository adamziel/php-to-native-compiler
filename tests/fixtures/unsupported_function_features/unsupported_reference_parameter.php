<?php
function mutate(&$value) {
    return $value;
}
$value = 1;
echo mutate($value);
