<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
    }

    public function connect_with_init_command($command) {
        mysqli_options($this->dbh, MYSQLI_INIT_COMMAND, $command);
        return mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }
}

$wpdb = new wpdb();
$wpdb->connect_with_init_command("SELECT 1");
