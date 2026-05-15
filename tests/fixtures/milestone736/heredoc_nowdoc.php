<?php
$name = "Ada";
$items = ["code" => 42];

$xml = <<<XML
<error>
    <title>{$name}</title>
    <code>{$items['code']}</code>
</error>
XML;

$literal = <<<'TXT'
literal $name
TXT;

echo $xml, $literal;

