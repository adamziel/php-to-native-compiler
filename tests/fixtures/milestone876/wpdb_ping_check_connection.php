<?php
class wpdb {
    public $dbh;
    public $checked;
    public $ready;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->checked = false;
        $this->ready = false;
    }

    public function check_connection() {
        $this->checked = true;

        if (mysqli_ping($this->dbh)) {
            $this->ready = true;
            return true;
        }

        $this->ready = false;
        return false;
    }
}

$wpdb = new wpdb();
echo $wpdb->check_connection() ? "alive" : "down";
echo "\n";
echo $wpdb->checked ? "checked" : "skipped";
echo "\n";
echo $wpdb->ready ? "ready" : "not-ready";
