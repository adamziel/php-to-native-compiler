<?php
class Factory {
    public function &make_entry($value) {
        return $value;
    }
}

echo "registered";
