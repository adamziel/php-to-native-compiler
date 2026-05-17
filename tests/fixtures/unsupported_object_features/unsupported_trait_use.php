<?php
class Box {
    use Labels, OtherLabels, ThirdLabels {
        label insteadof OtherLabels;
    }
}
