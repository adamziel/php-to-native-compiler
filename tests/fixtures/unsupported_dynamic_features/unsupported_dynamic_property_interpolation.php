<?php
class Partial {
    public $id;
}
$partial = new Partial();
$property = "id";
echo "customize_partial_render_{$partial->{$property}}";
