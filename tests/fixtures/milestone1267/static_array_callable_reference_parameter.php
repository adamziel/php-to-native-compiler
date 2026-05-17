<?php
class WP_Object_Cache {
    public $cache = [];
}

class Cache_Marker {
    public static function tag(&$value, $suffix) {
        $value = $value . ':' . $suffix;
        return $value;
    }
}

$cache = new WP_Object_Cache();
$cache->cache['options']['alloptions'] = 'cold';
$option = 'start';
echo call_user_func_array(array('Cache_Marker', 'tag'), array(&$option, 'direct')), "\n";
call_user_func_array(array('Cache_Marker', 'tag'), array(&$cache->cache['options']['alloptions'], 'static'));
echo $option, '|', $cache->cache['options']['alloptions'];
