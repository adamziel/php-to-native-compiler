<?php
class Catalog {
    public $entries;
}
$catalog = new Catalog();
$key = "name";
$entry = "Grace";
$catalog->entries[$key] =& $entry;
echo $catalog->entries["name"], "|";
$entry = "Hedy";
echo $catalog->entries["name"], "|";
$catalog->entries["name"] = "Katherine";
echo $entry, "|";
unset($entry);
$entry = "detached";
echo $catalog->entries["name"], "|", $entry;
echo "|";
$root = new Catalog();
$value = "root";
$root->entries["slot"] =& $value;
$value = "changed";
echo $root->entries["slot"], "|";
$root->entries["slot"] = "from-slot";
echo $value;
