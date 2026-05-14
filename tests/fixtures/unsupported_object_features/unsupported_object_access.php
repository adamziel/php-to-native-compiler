<?php
class Box {
    public function name() {}
}
$box = new Box();
$method = "name";
$box->$method();
