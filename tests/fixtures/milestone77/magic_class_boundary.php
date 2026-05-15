<?php
class Box {
    public function label() {
        return __CLASS__;
    }
}
$box = new Box();
echo $box->label();
