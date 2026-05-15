<?php
class wpdb {
    public $dbh;
    public $ready;
    public $closed;
    public $close_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->ready = true;
        $this->closed = false;
        $this->close_checked = false;
    }

    public function close() {
        $closed = mysqli_close($this->dbh);
        $this->closed = $closed;
        $this->ready = !$closed;
        $this->close_checked = true;

        return $closed;
    }
}

$wpdb = new wpdb();
echo $wpdb->close() ? "closed" : "open";
echo "\n";
echo $wpdb->closed ? "recorded" : "missing";
echo "\n";
echo $wpdb->ready ? "ready" : "not-ready";
echo "\n";
echo $wpdb->close_checked ? "checked" : "skipped";
