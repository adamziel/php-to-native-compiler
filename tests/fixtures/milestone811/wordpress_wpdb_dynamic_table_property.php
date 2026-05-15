<?php
class wpdb {}

$db = new wpdb();
$table = 'categories';
$db->$table = 'wp_categories';
echo $db->categories;
echo '|';
echo $db->$table;
