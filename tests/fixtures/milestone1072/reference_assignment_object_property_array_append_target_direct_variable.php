<?php
class Catalog {
    public $entries;
    public $groups;
}
$catalog = new Catalog();
$entry = "Grace";
$catalog->entries[] =& $entry;
echo $catalog->entries[0], "|";
$entry = "Hedy";
echo $catalog->entries[0], "|";
$catalog->entries[0] = "Katherine";
echo $entry, "|";
unset($entry);
$entry = "detached";
echo $catalog->entries[0], "|", $entry;
echo "|";
$nested = "Ada";
$catalog->groups["names"][] =& $nested;
echo $catalog->groups["names"][0], "|";
$nested = "Lovelace";
echo $catalog->groups["names"][0], "|";
$catalog->groups["names"][0] = "Byron";
echo $nested, "|";
unset($nested);
$nested = "detached-nested";
echo $catalog->groups["names"][0], "|", $nested;
echo "|";
$label = "main";
$catalog->groups["labels"]["primary"] =& $label;
$label = "changed";
echo $catalog->groups["labels"]["primary"], "|";
$catalog->groups["labels"]["primary"] = "from-slot";
echo $label;
