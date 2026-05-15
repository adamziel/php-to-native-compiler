<?php
echo preg_replace('#^www\.#', '', 'www.example.test');
echo '|';
echo preg_replace('#^www\.#', '', 'mail.example.test');
echo '|';
echo preg_replace('#^www\.#', '', 'www2.example.test');
