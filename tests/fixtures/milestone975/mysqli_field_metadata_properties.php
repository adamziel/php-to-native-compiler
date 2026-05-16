<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");

$fields = mysqli_fetch_fields($result);
echo $fields[0]->name, ":", $fields[0]->orgname, ":", $fields[0]->table;
echo ":", $fields[0]->type, ":", $fields[0]->length, ":", $fields[0]->charsetnr;
echo "|";

$field = mysqli_fetch_field_direct($result, 1);
echo $field->name, ":", $field->orgtable, ":", $field->db, ":", $field->catalog;
echo ":", $field->type, ":", $field->max_length, ":", $field->decimals;
