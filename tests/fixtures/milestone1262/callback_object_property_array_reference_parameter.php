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

function tag_cache(&$value, $suffix) {
    $value = $value . ':' . $suffix;
    return $value;
}

$cache = new WP_Object_Cache();
$cache->cache['options']['alloptions'] = 'cold';
call_user_func_array('tag_cache', array(&$cache->cache['options']['alloptions'], 'function'));
$filter = new Cache_Filter();
echo call_user_func_array(array($filter, 'mark'), array(&$cache->cache['options']['alloptions'], 'method')), "\n";
echo $cache->cache['options']['alloptions'];
