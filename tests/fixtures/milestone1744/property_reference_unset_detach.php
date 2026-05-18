<?php
class DetachBox {
    public $item = "seed";
    public int $typed = 7;
    private $hidden = "hidden";

    public function clearHidden() {
        $alias =& $this->hidden;
        unset($this->hidden);
        echo isset($this->hidden) ? "hidden-set" : "hidden-unset", "|", $alias, "\n";
        $alias = "hidden-alias";
        echo isset($this->hidden) ? "hidden-set" : "hidden-unset", "|", $alias;
    }
}

$box = new DetachBox();
$alias =& $box->item;
unset($box->item);
echo isset($box->item) ? "item-set" : "item-unset", "|", $alias, "\n";
$alias = "alias";
echo isset($box->item) ? "item-set:" . $box->item : "item-unset", "|", $alias, "\n";

$typed =& $box->typed;
unset($box->typed);
echo isset($box->typed) ? "typed-set" : "typed-unset", "|", $typed, "\n";
$typed = "bad";
echo isset($box->typed) ? "typed-set:" . $box->typed : "typed-unset", "|", $typed, "\n";

$cloneBox = new DetachBox();
$cloneAlias =& $cloneBox->item;
$clone = clone $cloneBox;
unset($clone->item);
echo $cloneBox->item, "|", $cloneAlias, "|", (isset($clone->item) ? "clone-set" : "clone-unset"), "\n";
$cloneAlias = "clone-alias";
echo $cloneBox->item, "|", $cloneAlias, "|", (isset($clone->item) ? "clone-set" : "clone-unset"), "\n";

$privateBox = new DetachBox();
$privateBox->clearHidden();
