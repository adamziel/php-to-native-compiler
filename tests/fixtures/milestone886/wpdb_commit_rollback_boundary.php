<?php
class wpdb {
    public $dbh;
    public $transaction_events;
    public $last_transaction;
    public $committed;
    public $rolled_back;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->transaction_events = 0;
        $this->last_transaction = "";
        $this->committed = false;
        $this->rolled_back = false;
    }

    public function begin_transaction($name) {
        if (mysqli_begin_transaction($this->dbh, 0, $name)) {
            $this->last_transaction = $name;
            $this->transaction_events = $this->transaction_events + 1;
            return true;
        }

        return false;
    }

    public function commit_transaction($name) {
        if (mysqli_commit($this->dbh, 0, $name)) {
            $this->committed = true;
            $this->last_transaction = $name;
            $this->transaction_events = $this->transaction_events + 1;
            return true;
        }

        return false;
    }

    public function rollback_transaction($name) {
        if (mysqli_rollback($this->dbh, 0, $name)) {
            $this->rolled_back = true;
            $this->last_transaction = $name;
            $this->transaction_events = $this->transaction_events + 1;
            return true;
        }

        return false;
    }
}

$wpdb = new wpdb();
echo $wpdb->begin_transaction("wp-commit") ? "begin-commit" : "failed";
echo "\n";
echo $wpdb->commit_transaction("wp-commit") ? "committed" : "failed";
echo "\n";
echo $wpdb->begin_transaction("wp-rollback") ? "begin-rollback" : "failed";
echo "\n";
echo $wpdb->rollback_transaction("wp-rollback") ? "rolled-back" : "failed";
echo "\n";
echo $wpdb->committed ? "commit-checked" : "commit-skipped";
echo "\n";
echo $wpdb->rolled_back ? "rollback-checked" : "rollback-skipped";
echo "\n";
echo $wpdb->last_transaction;
echo "\n";
echo $wpdb->transaction_events;
