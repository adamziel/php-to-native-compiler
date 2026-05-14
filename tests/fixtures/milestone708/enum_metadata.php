<?php
namespace App\State;

class Box {}
interface Renderable {}
trait Logs {}
enum Mode { case Front; }
enum Status {}

echo enum_exists('App\\State\\Mode') ? "yes" : "no", "\n";
echo enum_exists('App\\State\\Status') ? "yes" : "no", "\n";
echo class_exists('App\\State\\Mode') ? "class-like" : "not-class", "\n";
echo interface_exists('App\\State\\Mode') ? "interface" : "not-interface", "\n";
echo trait_exists('App\\State\\Mode') ? "trait" : "not-trait", "\n";
print_r(get_declared_classes());
