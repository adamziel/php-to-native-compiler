<?php
$left = [2 => "two", "a" => "left", "b" => "keep"];
$middle = ["a" => "middle", 9 => "nine"];
$result = ["start", ...$left, "a" => "explicit", ...$middle, "tail"];
$lines = [];
foreach ($result as $key => $value) {
    $lines[] = $key . "=" . $value;
}
echo implode("\n", $lines);
