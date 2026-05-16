<?php
class wpdb {
    public $dbh;
    public $refresh_flags;
    public $refresh_result;
    public $dynamic_refresh_result;
    public $still_connected;
    public $refresh_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->refresh_flags = 0;
        $this->refresh_result = false;
        $this->dynamic_refresh_result = false;
        $this->still_connected = false;
        $this->refresh_checked = false;
    }

    public function record_refresh() {
        $refresh = "mysqli_refresh";
        $this->refresh_flags = MYSQLI_REFRESH_LOG | MYSQLI_REFRESH_TABLES;
        $this->refresh_result = mysqli_refresh($this->dbh, $this->refresh_flags);
        $this->dynamic_refresh_result = $refresh(
            $this->dbh,
            MYSQLI_REFRESH_STATUS | MYSQLI_REFRESH_THREADS | MYSQLI_REFRESH_REPLICA
        );
        $this->still_connected = mysqli_ping($this->dbh);
        $this->refresh_checked = true;

        return $this->refresh_result && $this->dynamic_refresh_result && $this->still_connected ? 0 : 1;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_refresh();
echo "\n";
echo $wpdb->refresh_flags;
echo "\n";
echo MYSQLI_REFRESH_REPLICA === MYSQLI_REFRESH_SLAVE ? "replica-alias" : "different";
echo "\n";
echo $wpdb->refresh_result ? "refreshed" : "failed";
echo "\n";
echo $wpdb->dynamic_refresh_result ? "dynamic" : "failed";
echo "\n";
echo $wpdb->still_connected ? "still-open" : "closed";
echo "\n";
echo $wpdb->refresh_checked ? "checked" : "skipped";
