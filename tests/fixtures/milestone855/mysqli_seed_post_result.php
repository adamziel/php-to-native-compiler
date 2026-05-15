<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo get_class($result);
echo "|";
echo mysqli_num_fields($result);
$field = mysqli_fetch_field($result);
echo "|", $field->name;
$field = mysqli_fetch_field($result);
echo "|", $field->name;
echo "|";
echo mysqli_fetch_field($result) === false ? "no-field" : "field";
$row = mysqli_fetch_object($result);
echo "|", get_class($row);
echo "|", $row->ID;
echo "|", $row->post_title;
echo "|";
echo mysqli_fetch_object($result) === false ? "no-row" : "row";
