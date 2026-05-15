<?php
class EntryFactory {
    public function make() {
        return 1;
    }

    public function run() {
        $entry =& $this->make();
    }
}

$factory = new EntryFactory();
$factory->run();
