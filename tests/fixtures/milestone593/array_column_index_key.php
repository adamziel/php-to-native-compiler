<?php
class Person {
    public $id;
    public $name;
}

$person = new Person();
$person->id = "p1";
$person->name = "Linus";

$rows = [];
$rows[] = ["id" => 10, "name" => "Ada"];
$rows[] = ["id" => "10", "name" => "Grace"];
$rows[] = ["name" => "NoId"];
$rows[] = ["id" => "code", "name" => null];
$rows[] = $person;
$rows[] = 42;

$indexed = array_column($rows, "name", "id");
print_r($indexed);

$call = "array_column";
$whole = $call($rows, null, "id");
echo count($whole), "|", $whole[10]["name"], "|", $whole[11]["name"], "|", $whole["p1"]->name, "|";
if ($whole["code"]["name"] === null) {
    echo "null";
}
