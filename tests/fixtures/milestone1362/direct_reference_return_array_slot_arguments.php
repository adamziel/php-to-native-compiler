<?php
function &wp_refcow_tag(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$_REQUEST["payload"] = ["slot" => "request"];
$request_alias =& wp_refcow_tag($_REQUEST["payload"]["slot"], "function");
$request_alias = $request_alias . ":alias";
echo $_REQUEST["payload"]["slot"], "|", $request_alias, "\n";

$items = ["outer" => ["slot" => "array"]];
$array_alias =& wp_refcow_tag($items["outer"]["slot"], "function");
$array_alias = $array_alias . ":alias";
echo $items["outer"]["slot"], "|", $array_alias, "\n";

class WP_Object_Cache {
    public $cache = [];
}

$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
$cache_alias =& wp_refcow_tag($cache->cache["options"]["alloptions"], "function");
$cache_alias = $cache_alias . ":alias";
echo $cache->cache["options"]["alloptions"], "|", $cache_alias;
