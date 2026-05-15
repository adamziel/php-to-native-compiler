<?php
class wpdb {
    public $dbh;
    public $last_field_count;
    public $metadata_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_field_count = -1;
        $this->metadata_checked = false;
    }

    public function record_field_count() {
        $this->last_field_count = mysqli_field_count($this->dbh);
        $this->metadata_checked = true;

        return $this->last_field_count;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_field_count();
echo "\n";
echo $wpdb->last_field_count;
echo "\n";
echo $wpdb->metadata_checked ? "checked" : "skipped";
