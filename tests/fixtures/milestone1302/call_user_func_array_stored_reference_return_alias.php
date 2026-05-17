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

$option = "autoload";
$args = [];
$args[0] =& $option;
$args[1] = "function";
$alias =& call_user_func_array("wp_refcow_tag", $args);
$alias = $alias . ":alias";
echo $option, "|", $args[0], "|", $alias, "\n";

$_REQUEST["mode"] = "draft";
$request_alias =& $_REQUEST["mode"];
$request_args = [];
$request_args[0] =& $request_alias;
$request_args[1] = "request";
$request_result =& call_user_func_array("wp_refcow_tag", $request_args);
$request_result = $request_result . ":seen";
echo $_REQUEST["mode"], "|", $request_args[0], "|", $request_result, "\n";

$filter = new WP_Filter_Box();
$method_alias =& call_user_func_array(array($filter, "mark"), $args);
$option = "root";
echo $method_alias, "|", $alias, "\n";

$cache = new WP_Object_Cache();
$cache->cache["options"]["alloptions"] = "cold";
$cache_slot =& $cache->cache["options"]["alloptions"];
$static_args = [];
$static_args[0] =& $cache_slot;
$static_args[1] = "static";
$static_alias =& call_user_func_array(array("WP_Static_Filter_Box", "tag"), $static_args);
$static_alias = $static_alias . ":done";
echo $cache->cache["options"]["alloptions"], "|", $static_args[0], "|", $static_alias;
