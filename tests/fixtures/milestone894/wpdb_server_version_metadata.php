<?php
class wpdb {
    public $dbh;
    public $server_info;
    public $server_version;
    public $metadata_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->server_info = "";
        $this->server_version = 0;
        $this->metadata_checked = false;
    }

    public function record_server_version() {
        $this->server_info = mysqli_get_server_info($this->dbh);
        $this->server_version = mysqli_get_server_version($this->dbh);
        $this->metadata_checked = true;

        return $this->server_version;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_server_version();
echo "\n";
echo $wpdb->server_info;
echo "\n";
echo $wpdb->server_version;
echo "\n";
echo $wpdb->metadata_checked ? "checked" : "skipped";
