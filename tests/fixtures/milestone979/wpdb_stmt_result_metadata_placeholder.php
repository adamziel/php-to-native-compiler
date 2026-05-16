<?php
class wpdb {
    public $dbh;
    public $stmt;
    public $field_count;
    public $field_name;

    public function __construct() {
        $this->dbh = mysqli_init();
        $this->stmt = mysqli_prepare($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
    }

    public function record_statement_metadata() {
        $this->field_count = mysqli_stmt_field_count($this->stmt);
        $metadata = mysqli_stmt_result_metadata($this->stmt);
        $this->field_name = mysqli_fetch_field_direct($metadata, 1)->name;
        mysqli_stmt_free_result($this->stmt);
    }
}

$wpdb = new wpdb();
$wpdb->record_statement_metadata();
echo $wpdb->field_count;
echo "|";
echo $wpdb->field_name;
