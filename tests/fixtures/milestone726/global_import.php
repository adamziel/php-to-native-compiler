<?php
$value = "root";
function read_global() {
    global $value;
    return $value;
}
echo read_global(), "\n";
function write_global() {
    global $value;
    $value = $value . "-updated";
}
write_global();
echo $value, "\n";
