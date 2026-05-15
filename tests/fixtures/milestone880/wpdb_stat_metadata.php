<?php
class wpdb {
    public $dbh;
    public $server_status;
    public $status_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->server_status = "";
        $this->status_checked = false;
    }

    public function record_server_status() {
        $this->server_status = mysqli_stat($this->dbh);
        $this->status_checked = true;

        return $this->server_status;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_server_status(), "\n";
echo $wpdb->server_status, "\n";
echo $wpdb->status_checked ? "checked" : "skipped";
