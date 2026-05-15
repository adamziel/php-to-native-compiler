<?php
class wpdb {
    public $dbh;
    public $charset_name;
    public $metadata_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        mysqli_set_charset($this->dbh, "utf8mb4");
        $this->charset_name = "";
        $this->metadata_checked = false;
    }

    public function record_charset_name() {
        $this->charset_name = mysqli_character_set_name($this->dbh);
        $this->metadata_checked = true;

        return $this->charset_name;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_charset_name();
echo "\n";
echo $wpdb->charset_name;
echo "\n";
echo $wpdb->metadata_checked ? "checked" : "skipped";
