<?php
class wpdb {
    public $dbh;
    public $transaction_started;
    public $transaction_name;
    public $transaction_count;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->transaction_started = false;
        $this->transaction_name = "";
        $this->transaction_count = 0;
    }

    public function begin_transaction($name) {
        if (mysqli_begin_transaction($this->dbh, 0, $name)) {
            $this->transaction_started = true;
            $this->transaction_name = $name;
            $this->transaction_count = $this->transaction_count + 1;
            return true;
        }

        return false;
    }
}

$wpdb = new wpdb();
echo $wpdb->begin_transaction("wp-bootstrap") ? "started" : "failed";
echo "\n";
echo $wpdb->transaction_started ? "checked" : "skipped";
echo "\n";
echo $wpdb->transaction_name;
echo "\n";
echo $wpdb->transaction_count;
