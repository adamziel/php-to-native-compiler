<?php
$rows = [];
$rows[] = ["id" => 10, "name" => "Ada"];
$rows[] = ["id" => "10", "name" => "Grace"];
$rows[] = ["name" => "NoId"];
$rows[] = ["id" => "code", "name" => null];

$indexed = array_column($rows, "name", "id");
print_r($indexed);

$whole = array_column($rows, null, "id");
echo count($whole), "|", $whole[10]["name"], "|", $whole[11]["name"], "|";
if ($whole["code"]["name"] === null) {
    echo "null";
}
