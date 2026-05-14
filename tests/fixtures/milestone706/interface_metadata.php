<?php
namespace App\Contracts;

interface Hookable {}
interface Logger {
    public function write($message);
}

echo interface_exists('App\\Contracts\\Hookable') ? "yes" : "no", "\n";
echo interface_exists('App\\Contracts\\Logger') ? "yes" : "no", "\n";
echo class_exists('App\\Contracts\\Logger') ? "class" : "not-class", "\n";
print_r(get_declared_interfaces());
