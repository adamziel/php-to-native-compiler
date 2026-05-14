<?php
namespace App\Core;

function label() {
    return "ok";
}

echo function_exists("App\\Core\\label") ? "1" : "0";
echo function_exists("APP\\CORE\\LABEL") ? "1" : "0";
echo function_exists("label") ? "1" : "0";
