<?php
function wp_refcow_mark_non_public(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

function &wp_refcow_pick_non_public(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

class WP_RefCow_Non_Public_Store {
    private $privateStore = ["slot" => "private", "direct" => "direct"];
    protected $protectedStore = ["slot" => "protected"];

    public function probe($peer) {
        wp_refcow_mark_non_public($this->privateStore["direct"], "call");

        call_user_func_array("wp_refcow_mark_non_public", array(&$this->privateStore["slot"], "mark"));
        $alias =& call_user_func_array("wp_refcow_pick_non_public", array(&$this->privateStore["slot"], "pick"));
        $alias = $alias . ":alias";

        call_user_func_array("wp_refcow_mark_non_public", array(&$peer->protectedStore["slot"], "peer"));

        echo $this->privateStore["direct"], "|", $this->privateStore["slot"], "|", $alias, "|", $peer->protectedStore["slot"];
    }
}

$left = new WP_RefCow_Non_Public_Store();
$right = new WP_RefCow_Non_Public_Store();
$left->probe($right);
