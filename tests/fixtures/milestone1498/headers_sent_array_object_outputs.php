<?php
class HeaderSlots {
    public $file = "initial";
    public $line = -1;
    public $bag = array();
}
$out = array();
$bag = array("file" => "initial", "line" => -1);
$slots = new HeaderSlots();
$before = headers_sent($bag["file"], $slots->line);
$out[] = ($before ? "sent" : "open") . ":" . $bag["file"] . ":" . $slots->line;
echo "body";
$after = headers_sent($slots->file, $bag["line"]);
$out[] = ($after ? "sent" : "open") . ":" . basename($slots->file) . ":" . $bag["line"];
$nested = headers_sent($slots->bag["file"], $slots->bag["line"]);
$out[] = ($nested ? "sent" : "open") . ":" . basename($slots->bag["file"]) . ":" . $slots->bag["line"];
echo "|" . implode("|", $out);
