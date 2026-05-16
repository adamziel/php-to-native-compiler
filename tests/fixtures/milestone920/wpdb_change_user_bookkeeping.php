<?php
class wpdb {
    public $dbh;
    public $dbuser;
    public $dbname;
    public $change_result;
    public $null_database_result;
    public $still_connected;
    public $change_checked;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_options($this->dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->dbuser = "unset";
        $this->dbname = "unset";
        $this->change_result = false;
        $this->null_database_result = false;
        $this->still_connected = false;
        $this->change_checked = false;
    }

    public function record_user_change($user, $password, $database) {
        $this->dbuser = $user;
        $this->dbname = $database;
        $this->change_result = mysqli_change_user($this->dbh, $user, $password, $database);
        $this->null_database_result = mysqli_change_user($this->dbh, $user, $password, null);
        $this->still_connected = mysqli_ping($this->dbh);
        $this->change_checked = true;

        return $this->change_result && $this->null_database_result && $this->still_connected ? 0 : 1;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_user_change("wordpress", "secret", "wordpress");
echo "\n";
echo $wpdb->dbuser;
echo "\n";
echo $wpdb->dbname;
echo "\n";
echo $wpdb->change_result ? "changed" : "failed";
echo "\n";
echo $wpdb->null_database_result ? "changed-null-db" : "failed";
echo "\n";
echo $wpdb->still_connected ? "still-open" : "closed";
echo "\n";
echo $wpdb->change_checked ? "checked" : "skipped";
