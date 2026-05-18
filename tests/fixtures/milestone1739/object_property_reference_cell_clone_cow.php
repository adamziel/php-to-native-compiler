<?php
class Milestone1739_Box {
    public $items = ["slot" => "public-original"];
    private $hidden = ["slot" => "private-original"];

    public function exercisePrivate() {
        $hidden =& $this->hidden;
        $copy = clone $this;
        $copy->hidden["slot"] = "private-clone";
        echo $hidden["slot"], "|", $this->hidden["slot"], "|", $copy->hidden["slot"], "\n";
    }
}

function milestone1739_box() {
    static $box;
    if (!$box) {
        $box = new Milestone1739_Box();
    }
    return $box;
}

$box = new Milestone1739_Box();
$items =& $box->items;
$target = [];
$target["copy"] =& $items;
$copy = clone $box;
$target["copy"]["slot"] = "public-target";
echo $items["slot"], "|", $box->items["slot"], "|", $copy->items["slot"], "|", $target["copy"]["slot"], "\n";
$copy->items["slot"] = "public-clone";
echo $items["slot"], "|", $box->items["slot"], "|", $copy->items["slot"], "|", $target["copy"]["slot"], "\n";

$dynamic = new Milestone1739_Box();
$property = "items";
$dynamicItems =& $dynamic->{$property};
$dynamicCopy = clone $dynamic;
$dynamicCopy->items["slot"] = "dynamic-clone";
echo $dynamicItems["slot"], "|", $dynamic->items["slot"], "|", $dynamicCopy->items["slot"], "\n";

$box->exercisePrivate();

$expressionItems =& milestone1739_box()->items;
$expressionCopy = clone milestone1739_box();
$expressionCopy->items["slot"] = "expression-clone";
echo $expressionItems["slot"], "|", milestone1739_box()->items["slot"], "|", $expressionCopy->items["slot"], "\n";

$expressionProperty = "items";
$expressionDynamicItems =& milestone1739_box()->{$expressionProperty};
$expressionDynamicCopy = clone milestone1739_box();
$expressionDynamicCopy->items["slot"] = "expression-dynamic-clone";
echo $expressionDynamicItems["slot"], "|", milestone1739_box()->items["slot"], "|", $expressionDynamicCopy->items["slot"], "\n";

$aliasBackedBox = new Milestone1739_Box();
$aliasBackedBox->items = ["root" => [], "source" => "alias-target-original"];
$aliasBackedRoot =& $aliasBackedBox->items["root"];
$aliasBackedSource = "alias-target-source";
$aliasBackedRoot["slot"] =& $aliasBackedSource;
$aliasBackedCopy = clone $aliasBackedBox;
$aliasBackedCopy->items["root"]["slot"] = "alias-target-clone";
echo $aliasBackedSource, "|", $aliasBackedRoot["slot"], "|", $aliasBackedBox->items["root"]["slot"], "|", $aliasBackedCopy->items["root"]["slot"];
