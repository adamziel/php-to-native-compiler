<?php
$storage = "initial";

class MagicBox {
    public function &__get($name) {
        global $storage;
        return $storage;
    }
}

$box = new MagicBox();
$alias =& $box->missing;
$alias = "from-alias";
echo $storage, "|";
$storage = "from-global";
echo $alias, "|";

$property = "dynamic";
$dynamic =& $box->$property;
$dynamic = "dynamic-alias";
echo $storage, "|";
$storage = "dynamic-global";
echo $dynamic;
