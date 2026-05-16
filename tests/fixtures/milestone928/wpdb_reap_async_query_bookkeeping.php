<?php
class wpdb {
    public $dbh;
    public $reap_result;
    public $still_connected;
    public $async_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->reap_result = "unset";
        $this->still_connected = false;
        $this->async_checked = false;
    }

    public function check_async_result() {
        $this->reap_result = mysqli_reap_async_query($this->dbh);
        $this->still_connected = mysqli_ping($this->dbh);
        $this->async_checked = true;

        return $this->reap_result === false && $this->still_connected ? 0 : 1;
    }
}

$wpdb = new wpdb();
echo $wpdb->check_async_result();
echo "\n";
echo $wpdb->reap_result === false ? "no-async" : "async";
echo "\n";
echo $wpdb->still_connected ? "still-open" : "closed";
echo "\n";
echo $wpdb->async_checked ? "checked" : "skipped";
