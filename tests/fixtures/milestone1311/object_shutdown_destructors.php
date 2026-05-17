<?php
class BaseLifecycle {
    public $name;

    public function __construct($name) {
        $this->name = $name;
        echo "construct:", $this->name, "\n";
    }

    public function __destruct() {
        echo "destruct:", $this->name;
        if ($this->name !== "first") {
            echo "\n";
        }
    }
}

class PluginLifecycle extends BaseLifecycle {}

$first = new PluginLifecycle("first");
$second = new PluginLifecycle("second");
$copy = clone $first;
$copy->name = "copy";
echo "body\n";
