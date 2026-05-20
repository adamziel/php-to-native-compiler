<?php
class Box {
    public $left;
    public $right;

    public function __get($name) {
        $left = $this->left;
        $right = $this->right;
        $merged = array_merge(array($left), array($right));
        return $merged[1];
    }
}

$box = new Box();
$box->left = array("ref" => array("v" => "left"));
$leftAlias =& $box->left["ref"]["v"];
$box->right = array("ref" => array("v" => "right"));
$rightAlias =& $box->right["ref"]["v"];
$tmp = $box->missing;
$tmp["ref"]["v"] = "updated";
echo $box->left["ref"]["v"], "\n";
echo $box->right["ref"]["v"];
