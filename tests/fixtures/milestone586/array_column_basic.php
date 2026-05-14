<?php
class Person {
    public $name;
    public $age;
    private $secret;
}

$ada = new Person();
$ada->name = "Ada";
$ada->age = 36;
$grace = new Person();
$grace->name = "Grace";
$grace->age = null;

$rows = [];
$rows["first"] = ["name" => "ArrayAda", "age" => 35];
$rows[] = $ada;
$rows[] = $grace;
$rows[] = ["age" => 99];
$rows[] = 42;

$names = array_column($rows, "name");
print_r($names);
$ages = array_column($rows, "age");
print_r($ages);
$whole = array_column($rows, null);
echo count($whole), "|", $whole[0]["name"], "|", $whole[1]->name, "|", $whole[4], "\n";

$call = "array_column";
$again = $call($rows, "name");
echo count($again), "|", $again[0], "|", $again[1], "|", $again[2], "\n";
$secrets = array_column($rows, "secret");
echo count($secrets);
