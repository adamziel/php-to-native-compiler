--TEST--
Foreach by-value copy-on-write mutations keep iteration set stable
--FILE--
<?php
$items = array("a" => "A", "b" => "B", "c" => "C");
foreach ($items as $key => $value) {
    echo $key, "=", $value, "\n";
    if ($key === "a") {
        unset($items["a"]);
        unset($items["b"]);
        $items[] = "D";
    }
    if ($key === "b") {
        unset($items["c"]);
    }
}
var_dump($items);

$source = array("x" => 1, "y" => 2);
$alias = $source;
foreach ($source as $key => $value) {
    echo $key, ":", $value, "\n";
    if ($key === "x") {
        $source["z"] = 3;
        $alias["y"] = 20;
    }
}
var_dump($source);
var_dump($alias);
$post = $source;
$post[] = 4;
var_dump($source);
var_dump($post);
?>
--EXPECT--
a=A
b=B
c=C
array(1) {
  [0]=>
  string(1) "D"
}
x:1
y:2
array(3) {
  ["x"]=>
  int(1)
  ["y"]=>
  int(2)
  ["z"]=>
  int(3)
}
array(2) {
  ["x"]=>
  int(1)
  ["y"]=>
  int(20)
}
array(3) {
  ["x"]=>
  int(1)
  ["y"]=>
  int(2)
  ["z"]=>
  int(3)
}
array(4) {
  ["x"]=>
  int(1)
  ["y"]=>
  int(2)
  ["z"]=>
  int(3)
  [0]=>
  int(4)
}
