<?php
class Catalog {
    public $entries;
}

$catalog = new Catalog();
$entry = "source";
$other =& $entry;
$catalog->entries["slot"] =& $entry;

$entry = "from-entry";
echo $catalog->entries["slot"], "|", $other, "\n";

$other = "from-other";
echo $catalog->entries["slot"], "|", $entry, "\n";

$catalog->entries["slot"] = "from-slot";
echo $entry, "|", $other;
