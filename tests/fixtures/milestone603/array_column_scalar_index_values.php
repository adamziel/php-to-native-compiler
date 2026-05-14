<?php
$rows = [];
$rows[] = ["id" => true, "name" => "true"];
$rows[] = ["id" => false, "name" => "false"];
$rows[] = ["id" => null, "name" => "null"];
$rows[] = ["id" => 1.0, "name" => "float"];
$rows[] = ["name" => "missing"];

$indexed = array_column($rows, "name", "id");
print_r($indexed);

$whole = array_column($rows, null, "id");
echo count($whole), "|", $whole[1]["name"], "|", $whole[0]["name"], "|", $whole[""]["name"], "|", $whole[2]["name"];
