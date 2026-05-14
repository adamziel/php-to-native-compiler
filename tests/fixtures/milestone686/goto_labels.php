<?php
echo "start\n";
goto done;
echo "skipped\n";
done:
echo "done\n";
while (true) {
    if (true) {
        goto flush_sub_part;
    }
    echo "never\n";
    flush_sub_part:
    echo "nested\n";
    break;
}
