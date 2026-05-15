<?php
class wpdb {
    public $dbh;
    public $thread_id;
    public $kill_result;
    public $unknown_kill_result;
    public $still_connected;
    public $thread_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->thread_id = 0;
        $this->kill_result = "unset";
        $this->unknown_kill_result = "unset";
        $this->still_connected = false;
        $this->thread_checked = false;
    }

    public function record_thread_lifecycle() {
        $this->thread_id = mysqli_thread_id($this->dbh);
        $this->kill_result = mysqli_kill($this->dbh, $this->thread_id);
        $this->unknown_kill_result = mysqli_kill($this->dbh, 99);
        $this->still_connected = mysqli_ping($this->dbh);
        $this->thread_checked = true;

        return $this->kill_result && !$this->unknown_kill_result && $this->still_connected ? 0 : 1;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_thread_lifecycle();
echo "\n";
echo $wpdb->thread_id;
echo "\n";
echo $wpdb->kill_result ? "killed-placeholder" : "kill-failed";
echo "\n";
echo $wpdb->unknown_kill_result ? "unexpected-thread" : "no-thread";
echo "\n";
echo $wpdb->still_connected ? "still-open" : "closed";
echo "\n";
echo $wpdb->thread_checked ? "checked" : "skipped";
