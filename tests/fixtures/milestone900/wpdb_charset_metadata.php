<?php
class wpdb {
    public $dbh;
    public $charset;
    public $collate;
    public $charset_number;
    public $metadata_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        mysqli_set_charset($this->dbh, "utf8mb4");
        $this->charset = "";
        $this->collate = "";
        $this->charset_number = 0;
        $this->metadata_checked = false;
    }

    public function record_charset_metadata() {
        $metadata = mysqli_get_charset($this->dbh);
        $this->charset = $metadata->charset;
        $this->collate = $metadata->collation;
        $this->charset_number = $metadata->number;
        $this->metadata_checked = true;

        return $metadata->max_length;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_charset_metadata();
echo "\n";
echo $wpdb->charset;
echo "\n";
echo $wpdb->collate;
echo "\n";
echo $wpdb->charset_number;
echo "\n";
echo $wpdb->metadata_checked ? "checked" : "skipped";
