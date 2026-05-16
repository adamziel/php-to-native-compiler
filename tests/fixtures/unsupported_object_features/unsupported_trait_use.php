<?php
class Box {
    use Labels, OtherLabels, ThirdLabels {
        Labels::label insteadof OtherLabels, ThirdLabels;
    }
}
