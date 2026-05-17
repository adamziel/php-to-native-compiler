<?php
function wp_refcow_mark_stored_non_public(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

function &wp_refcow_pick_stored_non_public(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

class WP_RefCow_Stored_Non_Public_Store {
    private $privateArgs = [];
    private $privateStore = ["slot" => "private"];
    protected $protectedArgs = [];
    protected $protectedStore = ["slot" => "protected"];

    public function probe($peer) {
        $privateSlot =& $this->privateStore["slot"];
        $this->privateArgs[0] =& $privateSlot;
        $this->privateArgs[1] = "mark";
        call_user_func_array("wp_refcow_mark_stored_non_public", $this->privateArgs);

        $this->privateArgs[1] = "pick";
        $alias =& call_user_func_array("wp_refcow_pick_stored_non_public", $this->privateArgs);
        $alias = $alias . ":alias";

        $protectedSlot =& $peer->protectedStore["slot"];
        $peer->protectedArgs[0] =& $protectedSlot;
        $peer->protectedArgs[1] = "peer";
        call_user_func_array("wp_refcow_mark_stored_non_public", $peer->protectedArgs);

        echo $this->privateStore["slot"], "|", $this->privateArgs[0], "|", $alias, "|", $peer->protectedStore["slot"], "|", $peer->protectedArgs[0];
    }
}

$left = new WP_RefCow_Stored_Non_Public_Store();
$right = new WP_RefCow_Stored_Non_Public_Store();
$left->probe($right);
