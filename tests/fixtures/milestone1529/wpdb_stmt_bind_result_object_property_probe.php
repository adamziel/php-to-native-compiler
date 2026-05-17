<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
mysqli_stmt_execute($stmt, array("1"));
mysqli_stmt_store_result($stmt);

class Row {
    public $ID;
    public $fields;
}
$row = new Row();
$row->fields = array();
$title_key = "post_title";

echo mysqli_stmt_bind_result($stmt, $row->ID, $row->fields[$title_key]) ? "bound" : "failed";
echo "|";
echo mysqli_stmt_fetch($stmt) ? $row->ID . ":" . $row->fields["post_title"] : "no-row";
echo "|";
$title_key = "changed";
echo mysqli_stmt_fetch($stmt) === null ? "done" : "again";
echo "|";
echo array_key_exists("changed", $row->fields) ? "changed" : "stable";
