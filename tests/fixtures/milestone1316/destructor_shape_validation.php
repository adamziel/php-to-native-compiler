<?php
class PluginLifecycle {
    public function __construct() {
        echo "construct\n";
    }

    protected function __destruct() {
        echo "unreachable\n";
    }
}

$object = new PluginLifecycle();
echo "body\n";
