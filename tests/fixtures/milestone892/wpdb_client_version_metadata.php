<?php
class wpdb {
    public $dbh;
    public $client_version;
    public $metadata_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->client_version = 0;
        $this->metadata_checked = false;
    }

    public function record_client_version() {
        $this->client_version = mysqli_get_client_version();
        $this->metadata_checked = true;

        return $this->client_version;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_client_version();
echo "\n";
echo $wpdb->client_version;
echo "\n";
echo $wpdb->metadata_checked ? "checked" : "skipped";
