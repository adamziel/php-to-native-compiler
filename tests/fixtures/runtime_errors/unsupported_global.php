<?php
$value = 1;
function read_global() {
    global $value;
    return $value;
}
echo read_global();
