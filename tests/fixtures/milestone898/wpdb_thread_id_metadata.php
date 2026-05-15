<?php
class wpdb {
    public $dbh;
    public $thread_id;
    public $metadata_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->thread_id = 0;
        $this->metadata_checked = false;
    }

    public function record_thread_id() {
        $this->thread_id = mysqli_thread_id($this->dbh);
        $this->metadata_checked = true;

        return $this->thread_id;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_thread_id();
echo "\n";
echo $wpdb->thread_id;
echo "\n";
echo $wpdb->metadata_checked ? "checked" : "skipped";
