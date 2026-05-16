<?php
class wpdb {
    public $dbh;
    public $prepared_sql;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->prepared_sql = "";
    }

    public function prepare_option_lookup($option_name) {
        $this->prepared_sql = "SELECT option_value FROM wp_options WHERE option_name = ?";
        return mysqli_prepare($this->dbh, $this->prepared_sql);
    }
}

$wpdb = new wpdb();
$wpdb->prepare_option_lookup("home");
