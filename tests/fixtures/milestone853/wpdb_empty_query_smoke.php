<?php
class wpdb {
    public $dbh;
    public $result;
    public $last_result;
    public $num_rows;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_result = [];
        $this->num_rows = 0;
    }

    public function _do_query($query) {
        $this->result = mysqli_query($this->dbh, $query);
    }

    public function query($query) {
        $this->last_result = [];
        $this->num_rows = 0;
        $this->_do_query($query);

        $row = mysqli_fetch_object($this->result);
        while ($row) {
            $this->last_result[] = $row;
            $this->num_rows++;
            $row = mysqli_fetch_object($this->result);
        }

        mysqli_free_result($this->result);
        while (mysqli_more_results($this->dbh)) {
            mysqli_next_result($this->dbh);
        }

        return $this->num_rows;
    }
}

$wpdb = new wpdb();
echo $wpdb->query("SELECT * FROM wp_posts WHERE 1 = 0"), "\n";
echo count($wpdb->last_result), "\n";
echo $wpdb->num_rows;
