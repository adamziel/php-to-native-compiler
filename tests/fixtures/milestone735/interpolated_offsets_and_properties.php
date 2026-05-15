<?php
$attributes = ["textAlign" => "center", 0 => "zero"];
$key = "textAlign";

class Partial {
    public $id;
}

$partial = new Partial();
$partial->id = "header";

echo "has-text-align-{$attributes['textAlign']}";
echo "|has-text-align-{$attributes[$key]}";
echo "|has-text-align-$attributes[textAlign]";
echo "|offset-{$attributes[0]}";
echo "|customize_partial_render_{$partial->id}";
