<?php
define("ARRAY_A", "ARRAY_A");

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

    public function get_results($query, $output = ARRAY_A) {
        $this->last_result = [];
        $this->num_rows = 0;
        $this->_do_query($query);

        if ($output === ARRAY_A) {
            $this->num_rows = mysqli_num_rows($this->result);
            $row = mysqli_fetch_assoc($this->result);
            while ($row) {
                $this->last_result[] = $row;
                $row = mysqli_fetch_assoc($this->result);
            }
        }

        mysqli_free_result($this->result);
        while (mysqli_more_results($this->dbh)) {
            mysqli_next_result($this->dbh);
        }

        return $this->last_result;
    }
}

$wpdb = new wpdb();
$empty = $wpdb->get_results("SELECT * FROM wp_posts WHERE 1 = 0", ARRAY_A);
echo $wpdb->num_rows, "\n";
echo count($empty), "\n";

$rows = $wpdb->get_results("SELECT ID, post_title FROM wp_posts WHERE ID = 1", ARRAY_A);
echo $wpdb->num_rows, "\n";
echo count($rows), "\n";
echo $rows[0]["ID"], "|", $rows[0]["post_title"];
