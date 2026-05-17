<?php
class WP_RefCow_Tagger {
    public $cache = [];

    public function &tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }

    public static function &tag_static(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }

    public function run_self(&$value) {
        $alias =& self::tag_static($value, "self");
        $alias = $alias . ":alias";
    }
}

$tagger = new WP_RefCow_Tagger();

$_REQUEST["payload"] = ["slot" => "request"];
$request_alias =& $tagger->tag($_REQUEST["payload"]["slot"], "method");
$request_alias = $request_alias . ":alias";
echo $_REQUEST["payload"]["slot"], "|", $request_alias, "\n";

$items = ["outer" => ["slot" => "array"]];
$array_alias =& WP_RefCow_Tagger::tag_static($items["outer"]["slot"], "static");
$array_alias = $array_alias . ":alias";
echo $items["outer"]["slot"], "|", $array_alias, "\n";

$tagger->cache["options"]["alloptions"] = "cold";
$cache_alias =& $tagger->tag($tagger->cache["options"]["alloptions"], "method");
$cache_alias = $cache_alias . ":alias";
echo $tagger->cache["options"]["alloptions"], "|", $cache_alias, "\n";

$self_items = ["slot" => "self"];
$tagger->run_self($self_items["slot"]);
echo $self_items["slot"];
