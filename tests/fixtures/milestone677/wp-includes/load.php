<?php
function wp_loaded_label() {
    return "loaded";
}

class WP_Loaded {
    public static function name() {
        return "class:" . static::class;
    }
}
