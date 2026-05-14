<?php
class Base {
    protected function seal() {
        return "sealed";
    }
}

$base = new Base();
$base->seal();
