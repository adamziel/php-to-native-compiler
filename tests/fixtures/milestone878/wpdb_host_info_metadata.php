<?php
class wpdb {
    public $dbh;
    public $host_info;
    public $host_info_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->host_info = "";
        $this->host_info_checked = false;
    }

    public function record_host_info() {
        $this->host_info = mysqli_get_host_info($this->dbh);
        $this->host_info_checked = true;

        return $this->host_info;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_host_info(), "\n";
echo $wpdb->host_info, "\n";
echo $wpdb->host_info_checked ? "checked" : "skipped";
