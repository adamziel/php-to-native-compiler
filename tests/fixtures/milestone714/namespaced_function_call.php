<?php
namespace App\Core;

function label($name = "Ada") {
    return __FUNCTION__ . ":" . $name;
}

echo label(), "\n";
echo LABEL("Grace"), "\n";
echo strlen("abc");
