<?php
function mark_refcow_literal_private_dynamic(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

function &pick_refcow_literal_private_dynamic(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

class RefcowLiteralPrivateDynamicStore {
    private $args = [];
    private $items = ["slot" => "array"];
    private $flag = "inner";

    public function run() {
        $property = "args";
        $value = "seed";
        $this->{$property} = array(&$value, "private");
        call_user_func_array("mark_refcow_literal_private_dynamic", $this->args);
        echo $value, "|", $this->args[0], "\n";

        $alias =& call_user_func_array("pick_refcow_literal_private_dynamic", $this->args);
        $alias = $alias . ":alias";
        echo $value, "|", $this->args[0], "|", $alias, "\n";

        $this->{$property} = array(&$this->items["slot"], "copy");
        $copy = $this->args;
        $copy[0] = "copied";
        echo $this->items["slot"], "|", $this->args[0], "|", $copy[0], "\n";

        $flag = "flag";
        $alias =& $this->{$flag};
        $alias = "changed";
        echo $this->flag, "|", $this->{$flag}, "|", $alias;
    }
}

$store = new RefcowLiteralPrivateDynamicStore();
$store->run();
