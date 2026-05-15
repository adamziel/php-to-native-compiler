<?php
$attributes = ["layout" => ["columns" => 3]];

class Block {
    public $context;
}

$block = new Block();
$block->context = ["displayLayout" => ["columns" => 4]];

echo "columns-{$attributes['layout']['columns']}";
echo "|columns-{$block->context['displayLayout']['columns']}";

