<?php
class Box {
    use Labels {
        Labels::label insteadof OtherLabels;
    }
}
