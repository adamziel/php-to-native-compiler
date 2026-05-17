<?php
class WP_RefCow_Context_Dynamic_Property_ArrayAccess_Bag implements ArrayAccess {
    private $items = ["slot" => "seed", "outer" => ["slot" => "nested"]];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class WP_RefCow_Context_Dynamic_Property_ArrayAccess_Holder_Base {
    protected $protectedBag;
}

class WP_RefCow_Context_Dynamic_Property_ArrayAccess_Holder extends WP_RefCow_Context_Dynamic_Property_ArrayAccess_Holder_Base {
    private $privateBag;

    public function exercise() {
        $private = "privateBag";
        $protected = "protectedBag";
        $this->privateBag = new WP_RefCow_Context_Dynamic_Property_ArrayAccess_Bag();
        $this->protectedBag = new WP_RefCow_Context_Dynamic_Property_ArrayAccess_Bag();

        $alias =& $this->{$private}["slot"];
        $alias = $alias . ":alias";
        echo $this->{$private}["slot"], "|", $alias, "\n";

        echo call_user_func_array(
            "wp_refcow_context_dynamic_property_array_access_mark",
            array(&$this->{$private}["outer"]["slot"], "callback")
        ), "|", $this->{$private}["outer"]["slot"], "\n";

        $stored = [];
        $stored["value"] =& $this->{$protected}["created"]["leaf"];
        $stored["suffix"] = "stored";
        call_user_func_array("wp_refcow_context_dynamic_property_array_access_mark", $stored);
        echo $this->{$protected}["created"]["leaf"], "|", $stored["value"], "\n";

        $picked =& call_user_func_array(
            "wp_refcow_context_dynamic_property_array_access_pick",
            array(&$this->{$protected}["return"], "pick")
        );
        $picked = $picked . ":picked";
        echo $this->{$protected}["return"], "|", $picked;
    }
}

function wp_refcow_context_dynamic_property_array_access_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_context_dynamic_property_array_access_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

$holder = new WP_RefCow_Context_Dynamic_Property_ArrayAccess_Holder();
$holder->exercise();
