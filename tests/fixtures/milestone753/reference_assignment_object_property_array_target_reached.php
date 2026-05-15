<?php
class Catalog {
    public $entries;

    public function run() {
        $entry = 1;
        $this->entries[$entry] =& $entry;
    }
}

$catalog = new Catalog();
$catalog->run();
