<?php
class Cache_Filter {
    public function &mark(&$value, $suffix) {
        $value = $value . ':' . $suffix;
        return $value;
    }
}

class Cache_Marker {
    public static function &tag(&$value, $suffix) {
        $value = $value . ':' . $suffix;
        return $value;
    }
}

class WP_Object_Cache {
    public $cache = [];
}

function &tag_option(&$value, $suffix) {
    $value = $value . ':' . $suffix;
    return $value;
}

$option = 'autoload';
$args = [];
$args[0] =& $option;
$args[1] = 'stored';
echo call_user_func_array('tag_option', $args), '|', $option, '|', $args[0], "\n";

$copy = $args;
$copy[1] = 'method';
$filter = new Cache_Filter();
echo call_user_func_array(array($filter, 'mark'), $copy), '|', $option, "\n";

$_REQUEST['mode'] = 'draft';
$request_alias =& $_REQUEST['mode'];
$request_args = [];
$request_args[0] =& $request_alias;
$request_args[1] = 'request';
echo call_user_func_array('tag_option', $request_args), '|', $_REQUEST['mode'], '|', $request_args[0], "\n";

$cache = new WP_Object_Cache();
$cache->cache['options']['alloptions'] = 'cold';
$cache_slot =& $cache->cache['options']['alloptions'];
$static_args = [];
$static_args[0] =& $cache_slot;
$static_args[1] = 'static';
echo call_user_func_array(array('Cache_Marker', 'tag'), $static_args), '|', $cache->cache['options']['alloptions'], "\n";

$cache->cache['options']['runtime'] = 'warm';
echo call_user_func_array('tag_option', array(&$cache->cache['options']['runtime'], 'literal')), '|', $cache->cache['options']['runtime'];
