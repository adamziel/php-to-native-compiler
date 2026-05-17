<?php
class WP_Object_Cache {
    public $cache = [];
}

class WP_Filter_Box {
    public function &mark(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

class WP_Static_Filter_Box {
    public static function &tag(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

function &wp_refcow_tag(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
$alias =& call_user_func_array("wp_refcow_tag", array(&$cache->cache["options"]["alloptions"], "function"));
$alias = $alias . ":alias";
echo $cache->cache["options"]["alloptions"], "|", $alias, "\n";

$filter = new WP_Filter_Box();
$method_alias =& call_user_func_array(array($filter, "mark"), array(&$cache->cache["options"]["alloptions"], "method"));
$cache->cache["options"]["alloptions"] = "root";
echo $method_alias, "|", $alias, "\n";

$static_alias =& call_user_func_array(array("WP_Static_Filter_Box", "tag"), array(&$cache->cache["options"]["alloptions"], "static"));
$static_alias = $static_alias . ":done";
echo $cache->cache["options"]["alloptions"], "|", $method_alias, "|", $static_alias;
