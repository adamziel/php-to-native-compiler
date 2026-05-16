<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
    }

    public function load_local_file() {
        mysqli_options($this->dbh, MYSQLI_OPT_LOCAL_INFILE, true);
        return mysqli_query($this->dbh, "LOAD DATA LOCAL INFILE '/tmp/posts.csv' INTO TABLE wp_posts");
    }
}

$wpdb = new wpdb();
$wpdb->load_local_file();
