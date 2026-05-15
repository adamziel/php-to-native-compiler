<?php
$ctx = [];
for ($i = 0; $i < 2; $i = $i + 1) {
    $ctx[0][$i] = $i + 10;
}
echo $ctx[0][0], ":", $ctx[0][1];
