<?php
$source = "seed";
$items = ["box" => ["leaf" => &$source]];
$alias =& $items["box"];
unset($items);
$alias["leaf"] = "mutated";
echo $source, "|", $alias["leaf"];
