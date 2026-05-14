<?php
if (PHP_VERSION_ID >= 70000) {
    echo spl_autoload_register(function ($class) {
        return false;
    }) ? "registered" : "failed";
}
