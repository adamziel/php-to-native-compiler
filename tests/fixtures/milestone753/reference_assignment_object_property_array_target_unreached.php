<?php
class Catalog {
    public $entries;

    public function register() {
        $entry = 1;
        $this->entries[$entry] =& $entry;
    }
}

echo "registered";
