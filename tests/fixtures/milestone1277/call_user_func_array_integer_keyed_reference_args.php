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
echo call_user_func_array('tag_option', array(10 => &$option, 20 => 'function')), "\n";

$filter = new Cache_Filter();
echo call_user_func_array(array($filter, 'mark'), array(2 => &$option, 7 => 'method')), "\n";

$cache = new WP_Object_Cache();
$cache->cache['options']['alloptions'] = 'cold';
call_user_func_array(
    array('Cache_Marker', 'tag'),
    array(5 => &$cache->cache['options']['alloptions'], 8 => 'static')
);

echo $option, '|', $cache->cache['options']['alloptions'];
