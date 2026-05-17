<?php
class WP_Object_Cache {
    public $cache = [];
}

class Parent_Cache_Marker {
    public function mark_parent(&$value, $suffix) {
        $value = $value . ':parent-' . $suffix;
    }

    public static function tag_parent(&$value, $suffix) {
        $value = $value . ':parent-static-' . $suffix;
    }
}

class Child_Cache_Marker extends Parent_Cache_Marker {
    public function mark($cache) {
        parent::mark_parent($cache->cache['options']['alloptions'], 'method');
        parent::tag_parent($cache->cache['options']['alloptions'], 'method');
    }
}

$cache = new WP_Object_Cache();
$cache->cache['options']['alloptions'] = 'cold';
$marker = new Child_Cache_Marker();
$marker->mark($cache);
echo $cache->cache['options']['alloptions'];
