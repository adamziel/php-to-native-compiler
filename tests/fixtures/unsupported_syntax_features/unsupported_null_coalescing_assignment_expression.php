<?php
$items = [];
echo ($items["outer"]["inner"] ??= "fallback");
