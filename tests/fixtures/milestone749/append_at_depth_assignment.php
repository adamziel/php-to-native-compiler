<?php
$submenu = [];
$submenu['themes.php'][] = ['Widgets', 'edit_theme_options', 'widgets.php'];
$submenu['themes.php'][] = ['Customize', 'edit_theme_options', 'customize.php'];

echo $submenu['themes.php'][0][0], "\n";
echo $submenu['themes.php'][1][2], "\n";

$created = [];
echo ($created['outer']['inner'][] = 'made'), "\n";
echo $created['outer']['inner'][0];
