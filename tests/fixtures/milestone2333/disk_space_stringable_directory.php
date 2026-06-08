<?php
class DiskPath {
    private $path;

    public function __construct($path) {
        $this->path = $path;
    }

    public function __toString() {
        return $this->path;
    }
}

class PlainDiskPath {}

$path = new DiskPath(__DIR__);
$alias = "diskfreespace";

echo is_float(disk_free_space($path)) ? "free-object\n" : "bad-free\n";
echo is_float($alias($path)) ? "alias-object\n" : "bad-alias\n";
echo is_float(disk_total_space($path)) ? "total-object\n" : "bad-total\n";

try {
    disk_free_space(new PlainDiskPath());
} catch (TypeError $e) {
    echo "plain-object-caught\n";
}

try {
    disk_total_space([]);
} catch (TypeError $e) {
    echo "array-caught\n";
}

try {
    diskfreespace(new DiskPath(__DIR__ . chr(0)));
} catch (ValueError $e) {
    echo $e->getMessage();
}
