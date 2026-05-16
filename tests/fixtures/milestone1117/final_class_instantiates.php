<?php
final class Base {
    public $label = "base";
}

$base = new Base();
echo get_class($base), ":", $base->label;
