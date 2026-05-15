<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "SELECT * FROM wp_posts WHERE ID = 1");
