<?php
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
$alias =& call_user_func_array("wp_refcow_tag", array(&$option, "function"));
$alias = $alias . ":alias";
echo $option, "|", $alias, "\n";

$filter = new WP_Filter_Box();
$method_alias =& call_user_func_array(array($filter, "mark"), array(&$option, "method"));
$option = "root";
echo $method_alias, "|", $alias, "\n";

$static_alias =& call_user_func_array(array("WP_Static_Filter_Box", "tag"), array(&$option, "static"));
$static_alias = $static_alias . ":done";
echo $option, "|", $method_alias, "|", $static_alias;
