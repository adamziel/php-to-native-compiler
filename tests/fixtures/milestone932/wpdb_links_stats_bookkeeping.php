<?php
class wpdb {
    public $links_stats;
    public $links_checked;

    public function __construct() {
        $this->links_stats = [];
        $this->links_checked = false;
    }

    public function record_links_stats() {
        $this->links_stats = mysqli_get_links_stats();
        $this->links_checked = true;

        return $this->links_stats["total"] === 0
            && $this->links_stats["active_plinks"] === 0
            && $this->links_stats["cached_plinks"] === 0 ? 0 : 1;
    }
}

$wpdb = new wpdb();
echo $wpdb->record_links_stats();
echo "\n";
echo $wpdb->links_stats["total"];
echo "\n";
echo $wpdb->links_stats["active_plinks"];
echo "\n";
echo $wpdb->links_stats["cached_plinks"];
echo "\n";
echo $wpdb->links_checked ? "checked" : "skipped";
