<?php
class Milestone1736_Box {
    public $items = [];
    private $hidden = [];

    public function bind(&$slot, &$append, &$dynamic, &$privateSlot, &$privateAppend, &$privateDynamic) {
        $property = "items";
        $hiddenProperty = "hidden";
        $this->items["slot"] =& $slot;
        $this->items["outer"][] =& $append;
        $this->{$property}["dynamic"] =& $dynamic;
        $this->hidden["slot"] =& $privateSlot;
        $this->hidden["outer"][] =& $privateAppend;
        $this->{$hiddenProperty}["dynamic"] =& $privateDynamic;
    }

    public function mutateHidden() {
        $this->hidden["slot"] = "private-slot-property";
        $this->hidden["outer"][0] = "private-append-property";
        $this->hidden["dynamic"] = "private-dynamic-property";
    }

    public function hiddenReport() {
        return $this->hidden["slot"] . "|" .
            $this->hidden["outer"][0] . "|" .
            $this->hidden["dynamic"];
    }
}

$slot = "slot-seed";
$append = "append-seed";
$dynamic = "dynamic-seed";
$privateSlot = "private-slot-seed";
$privateAppend = "private-append-seed";
$privateDynamic = "private-dynamic-seed";

$box = new Milestone1736_Box();
$box->bind($slot, $append, $dynamic, $privateSlot, $privateAppend, $privateDynamic);

$slot = "slot-variable";
$append = "append-variable";
$dynamic = "dynamic-variable";
$privateSlot = "private-slot-variable";
$privateAppend = "private-append-variable";
$privateDynamic = "private-dynamic-variable";

$box->items["slot"] = "slot-property";
$box->items["outer"][0] = "append-property";
$box->items["dynamic"] = "dynamic-property";
$box->mutateHidden();

echo $slot,
    "|",
    $append,
    "|",
    $dynamic,
    "|",
    $box->items["slot"],
    "|",
    $box->items["outer"][0],
    "|",
    $box->items["dynamic"],
    "|",
    $privateSlot,
    "|",
    $privateAppend,
    "|",
    $privateDynamic,
    "|",
    $box->hiddenReport();
