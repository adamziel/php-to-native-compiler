<?php
$handler = static function ($value) {
    echo "body";
    return $value;
};

echo $handler ? "stored" : "missing";
