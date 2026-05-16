<?php
$items = ["name" => "Ada", "other" => "Grace"];
$key = "name";
$alias =& $items[$key];
$alias = "Katherine";
echo $items["name"], "|", $alias;
echo "|";
$items[$key] = "Hedy";
echo $items["name"], "|", $alias, "|", $items["other"];
echo "|";
$missing =& $items["missing"];
$missing = "materialized";
echo $items["missing"];
