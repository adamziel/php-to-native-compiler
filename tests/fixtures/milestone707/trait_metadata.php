<?php
namespace App\Shared;

trait Hookable {}
trait Logs {}

echo trait_exists('App\\Shared\\Hookable') ? "yes" : "no", "\n";
echo trait_exists('App\\Shared\\Logs') ? "yes" : "no", "\n";
echo class_exists('App\\Shared\\Logs') ? "class" : "not-class", "\n";
echo interface_exists('App\\Shared\\Logs') ? "interface" : "not-interface", "\n";
print_r(get_declared_traits());
