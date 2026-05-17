<?php
class WP_Object_Cache {
    public $cache = [];

    public function tag(&$value, $suffix) {
        $value = $value . ':' . $suffix;
    }
}

function cache_mark(&$value, $suffix) {
    $value = $value . ':' . $suffix;
}

$cache = new WP_Object_Cache();
$cache->cache['options']['alloptions'] = 'cold';
cache_mark($cache->cache['options']['alloptions'], 'function');
$cache->tag($cache->cache['options']['alloptions'], 'method');
echo $cache->cache['options']['alloptions'];
