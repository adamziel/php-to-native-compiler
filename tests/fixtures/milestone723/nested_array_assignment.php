<?php
$items = [];
$items["outer"]["inner"] = "Ada";
echo $items["outer"]["inner"], "\n";

$created["a"]["b"] = "made";
echo $created["a"]["b"], "\n";

$deep = [];
echo ($deep["x"]["y"]["z"] = "deep"), ":", $deep["x"]["y"]["z"], "\n";
echo "done";
