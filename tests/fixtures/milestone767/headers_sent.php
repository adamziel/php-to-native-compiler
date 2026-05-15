<?php
header("X-Test: one");
echo headers_sent() ? "sent" : "open";
