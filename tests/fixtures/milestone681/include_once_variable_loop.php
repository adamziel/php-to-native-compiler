<?php
$count = 0;
$plugins = ['loop-lib.php', 'loop-lib.php'];

foreach ($plugins as $mu_plugin) {
    include_once $mu_plugin;
}

echo "loop-count=", $count;
