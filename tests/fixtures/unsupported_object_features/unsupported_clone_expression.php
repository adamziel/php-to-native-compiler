<?php
class Box {
    public function __clone() {
        echo "clone";
    }
}
$box = new Box();
$copy = clone $box;
