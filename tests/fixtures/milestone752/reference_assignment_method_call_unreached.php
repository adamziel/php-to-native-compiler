<?php
class EntryFactory {
    public function make() {
        return 1;
    }

    public function register() {
        $entry =& $this->make();
        return $entry;
    }
}

echo "registered";
