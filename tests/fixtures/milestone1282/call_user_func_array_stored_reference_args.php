<?php
class WP_Object_Cache {
    public $cache = [];
}

class Cache_Filter {
    public function mark(&$value, $suffix) {
        $value = $value . ':' . $suffix;
        return $value;
    }
}

class Cache_Marker {
    public static function tag(&$value, $suffix) {
        $value = $value . ':' . $suffix;
        return $value;
    }
}

function tag_option(&$value, $suffix) {
    $value = $value . ':' . $suffix;
    return $value;
}

$option = 'autoload';
$args = [];
$args[10] =& $option;
$args[20] = 'stored';
echo call_user_func_array('tag_option', $args), "\n";

$copy = $args;
$copy[20] = 'copy';
$filter = new Cache_Filter();
echo call_user_func_array(array($filter, 'mark'), $copy), "\n";

$_REQUEST['mode'] = 'draft';
$request_alias =& $_REQUEST['mode'];
$request_args = [];
$request_args[0] =& $request_alias;
$request_args[1] = 'request';
call_user_func_array('tag_option', $request_args);
echo $_REQUEST['mode'], '|', $request_args[0], "\n";

$cache = new WP_Object_Cache();
$cache->cache['options']['alloptions'] = 'cold';
$cache_slot =& $cache->cache['options']['alloptions'];
$static_args = [];
$static_args[0] =& $cache_slot;
$static_args[1] = 'static';
call_user_func_array(array('Cache_Marker', 'tag'), $static_args);

echo $option, '|', $cache->cache['options']['alloptions'];
