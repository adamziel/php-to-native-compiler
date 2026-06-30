#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#endif

#include <ctype.h>
#include <errno.h>
#include <float.h>
#include <limits.h>
#include <locale.h>
#include <math.h>
#include <stdarg.h>
#include <signal.h>
#include <setjmp.h>
#include <stdio.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#if defined(_WIN32)
#include <direct.h>
#include <io.h>
#include <process.h>
#include <sys/utime.h>
#else
#include <dirent.h>
#include <arpa/inet.h>
#include <dlfcn.h>
#include <fnmatch.h>
#include <grp.h>
#include <glob.h>
#include <iconv.h>
#include <langinfo.h>
#include <netdb.h>
#include <fcntl.h>
#include <pwd.h>
#include <regex.h>
#include <resolv.h>
#include <sys/file.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <sys/statvfs.h>
#include <sys/un.h>
#include <sys/utsname.h>
#include <sys/wait.h>
#include <ucontext.h>
#include <utime.h>
#include <unistd.h>
#endif
#ifdef PTN_USE_ADA_URL
#include "ada_c.h"
#endif

#ifndef PTN_HAVE_OPENSSL
#define PTN_HAVE_OPENSSL 0
#endif
#if PTN_HAVE_OPENSSL
#include <openssl/asn1.h>
#include <openssl/bio.h>
#include <openssl/buffer.h>
#include <openssl/cms.h>
#include <openssl/err.h>
#include <openssl/evp.h>
#include <openssl/objects.h>
#include <openssl/opensslv.h>
#include <openssl/pkcs12.h>
#include <openssl/pem.h>
#include <openssl/provider.h>
#include <openssl/rsa.h>
#include <openssl/x509.h>
#endif

#ifndef PTN_HAVE_ZLIB
#define PTN_HAVE_ZLIB 0
#endif
#if PTN_HAVE_ZLIB
#include <zlib.h>
#endif

#if !defined(_WIN32)
extern char *realpath(const char *path, char *resolved_path);
extern char *strptime(const char *s, const char *format, struct tm *tm);
extern char **environ;
#define PTN_ENVIRON environ
#else
extern char **_environ;
#define PTN_ENVIRON _environ
#endif

#if defined(__GNUC__) || defined(__clang__)
#define PTN_UNUSED __attribute__((unused))
#else
#define PTN_UNUSED
#endif

#ifndef R_OK
#define R_OK 4
#endif
#ifndef W_OK
#define W_OK 2
#endif
#ifndef X_OK
#define X_OK 1
#endif

#if defined(__GNUC__) && !defined(__clang__)
#pragma GCC diagnostic ignored "-Wclobbered"
#endif

#if defined(_WIN32)
#define REG_EXTENDED 0
#define REG_ICASE 0
typedef struct {
    size_t re_nsub;
} regex_t;
typedef struct {
    int rm_so;
    int rm_eo;
} regmatch_t;
#endif

#define PTN_PHP_VERSION "8.4.0"
#define PTN_PHP_MAJOR_VERSION 8
#define PTN_PHP_MINOR_VERSION 4
#define PTN_PHP_RELEASE_VERSION 0
#define PTN_PHP_EXTRA_VERSION ""
#define PTN_PHP_VERSION_ID 80400
#define PTN_PHP_ZTS 0
#define PTN_PHP_DEBUG 0
#define PTN_PHP_SAPI_NAME "cli"
#define PTN_ZEND_VERSION "4.4.0"
#define PTN_PHP_EXTENSION_DIR "."
#define PTN_PHP_BINARY "phpc"
#define PTN_DEFAULT_UNSERIALIZE_MAX_DEPTH 4096
#define PTN_INI_SCANNER_NORMAL 0
#define PTN_INI_SCANNER_RAW 1
#define PTN_INI_SCANNER_TYPED 2
#ifdef PATH_MAX
#define PTN_PHP_MAXPATHLEN PATH_MAX
#else
#define PTN_PHP_MAXPATHLEN 4096
#endif
#if defined(_WIN32)
#define PTN_PHP_OS "WINNT"
#define PTN_PHP_OS_FAMILY "Windows"
#define PTN_PHP_SHLIB_SUFFIX "dll"
#elif defined(__APPLE__)
#define PTN_PHP_OS "Darwin"
#define PTN_PHP_OS_FAMILY "Darwin"
#define PTN_PHP_SHLIB_SUFFIX "dylib"
#elif defined(__linux__)
#define PTN_PHP_OS "Linux"
#define PTN_PHP_OS_FAMILY "Linux"
#define PTN_PHP_SHLIB_SUFFIX "so"
#elif defined(__FreeBSD__)
#define PTN_PHP_OS "FreeBSD"
#define PTN_PHP_OS_FAMILY "BSD"
#define PTN_PHP_SHLIB_SUFFIX "so"
#else
#define PTN_PHP_OS "Unknown"
#define PTN_PHP_OS_FAMILY "Unknown"
#define PTN_PHP_SHLIB_SUFFIX "so"
#endif
#if PTN_HAVE_OPENSSL
#define PTN_OPENSSL_VERSION_NUMBER OPENSSL_VERSION_NUMBER
#define PTN_OPENSSL_VERSION_TEXT OPENSSL_VERSION_TEXT
#else
#define PTN_OPENSSL_VERSION_NUMBER 0
#define PTN_OPENSSL_VERSION_TEXT ""
#endif
#define PTN_ARRAY_INDEX_MIN_ENTRIES 16
#define PTN_SYMBOL_INDEX_MIN_ENTRIES 16
#define PTN_ARRAY_MAX_ALLOC_ENTRIES 1048576ULL
#define PTN_E_ERROR 1
#define PTN_E_WARNING 2
#define PTN_E_PARSE 4
#define PTN_E_NOTICE 8
#define PTN_E_CORE_ERROR 16
#define PTN_E_CORE_WARNING 32
#define PTN_E_COMPILE_ERROR 64
#define PTN_E_COMPILE_WARNING 128
#define PTN_E_USER_ERROR 256
#define PTN_E_USER_WARNING 512
#define PTN_E_USER_NOTICE 1024
#define PTN_E_STRICT 2048
#define PTN_E_RECOVERABLE_ERROR 4096
#define PTN_E_DEPRECATED 8192
#define PTN_E_USER_DEPRECATED 16384
#define PTN_E_ALL 30719
#define PTN_EXTR_OVERWRITE 0
#define PTN_EXTR_SKIP 1
#define PTN_EXTR_PREFIX_SAME 2
#define PTN_EXTR_PREFIX_ALL 3
#define PTN_EXTR_PREFIX_INVALID 4
#define PTN_EXTR_PREFIX_IF_EXISTS 5
#define PTN_EXTR_IF_EXISTS 6
#define PTN_EXTR_REFS 256
#define PTN_ARRAY_FILTER_USE_BOTH 1
#define PTN_ARRAY_FILTER_USE_KEY 2
#define PTN_FILTER_VALIDATE_REGEXP 272
#define PTN_SORT_REGULAR 0
#define PTN_SORT_NUMERIC 1
#define PTN_SORT_STRING 2
#define PTN_SORT_DESC 3
#define PTN_SORT_ASC 4
#define PTN_SORT_LOCALE_STRING 5
#define PTN_SORT_NATURAL 6
#define PTN_SORT_FLAG_CASE 8
#define PTN_SCANDIR_SORT_ASCENDING 0
#define PTN_SCANDIR_SORT_DESCENDING 1
#define PTN_SCANDIR_SORT_NONE 2
#define PTN_PHP_ROUND_HALF_UP 1
#define PTN_PHP_ROUND_HALF_DOWN 2
#define PTN_PHP_ROUND_HALF_EVEN 3
#define PTN_PHP_ROUND_HALF_ODD 4
#define PTN_PHP_ROUND_CEILING 5
#define PTN_PHP_ROUND_FLOOR 6
#define PTN_PHP_ROUND_TOWARD_ZERO 7
#define PTN_PHP_ROUND_AWAY_FROM_ZERO 8
#define PTN_PHP_OUTPUT_HANDLER_TYPE_INTERNAL 0
#define PTN_PHP_OUTPUT_HANDLER_TYPE_USER 1
#define PTN_PHP_OUTPUT_HANDLER_CLEANABLE 16
#define PTN_PHP_OUTPUT_HANDLER_FLUSHABLE 32
#define PTN_PHP_OUTPUT_HANDLER_REMOVABLE 64
#define PTN_PHP_OUTPUT_HANDLER_STDFLAGS 112
#define PTN_PHP_OUTPUT_HANDLER_STARTED 4096
#define PTN_PHP_OUTPUT_HANDLER_DISABLED 8192
#define PTN_PHP_OUTPUT_HANDLER_PROCESSED 16384
#define PTN_PHP_OUTPUT_HANDLER_WRITE 0
#define PTN_PHP_OUTPUT_HANDLER_START 1
#define PTN_PHP_OUTPUT_HANDLER_CLEAN 2
#define PTN_PHP_OUTPUT_HANDLER_FLUSH 4
#define PTN_PHP_OUTPUT_HANDLER_FINAL 8
#define PTN_HTML_SPECIALCHARS 0
#define PTN_HTML_ENTITIES 1
#define PTN_ENT_NOQUOTES 0
#define PTN_ENT_COMPAT 2
#define PTN_ENT_QUOTES 3
#define PTN_ENT_IGNORE 4
#define PTN_ENT_SUBSTITUTE 8
#define PTN_ENT_HTML401 0
#define PTN_ENT_XML1 16
#define PTN_ENT_XHTML 32
#define PTN_ENT_HTML5 48
#define PTN_ENT_DISALLOWED 128
#define PTN_ICONV_MIME_DECODE_STRICT 1
#define PTN_ICONV_MIME_DECODE_CONTINUE_ON_ERROR 2
#define PTN_SUNFUNCS_RET_TIMESTAMP 0
#define PTN_SUNFUNCS_RET_STRING 1
#define PTN_SUNFUNCS_RET_DOUBLE 2
#define PTN_CRYPT_SALT_LENGTH 123
#define PTN_CRYPT_STD_DES 1
#define PTN_CRYPT_EXT_DES 1
#define PTN_CRYPT_MD5 1
#define PTN_CRYPT_BLOWFISH 1
#define PTN_CRYPT_SHA256 1
#define PTN_CRYPT_SHA512 1
#define PTN_PASSWORD_BCRYPT "2y"
#define PTN_PASSWORD_LEGACY_BCRYPT 1
#define PTN_PASSWORD_BCRYPT_DEFAULT_COST 12
#define PTN_STR_PAD_LEFT 0
#define PTN_STR_PAD_RIGHT 1
#define PTN_STR_PAD_BOTH 2
#define PTN_COUNT_NORMAL 0
#define PTN_COUNT_RECURSIVE 1
#define PTN_INFO_GENERAL 1
#define PTN_INFO_CREDITS 2
#define PTN_INFO_CONFIGURATION 4
#define PTN_INFO_MODULES 8
#define PTN_INFO_ENVIRONMENT 16
#define PTN_INFO_VARIABLES 32
#define PTN_INFO_LICENSE 64
#define PTN_INFO_ALL 4294967295LL
#define PTN_CREDITS_GROUP 1
#define PTN_CREDITS_GENERAL 2
#define PTN_CREDITS_SAPI 4
#define PTN_CREDITS_MODULES 8
#define PTN_CREDITS_DOCS 16
#define PTN_CREDITS_FULLPAGE 32
#define PTN_CREDITS_QA 64
#define PTN_CREDITS_WEB 128
#define PTN_CREDITS_ALL 4294967295LL
#define PTN_PHP_SESSION_DISABLED 0
#define PTN_PHP_SESSION_NONE 1
#define PTN_PHP_SESSION_ACTIVE 2
#define PTN_PATHINFO_DIRNAME 1
#define PTN_PATHINFO_BASENAME 2
#define PTN_PATHINFO_EXTENSION 4
#define PTN_PATHINFO_FILENAME 8
#define PTN_PATHINFO_ALL 15
#define PTN_PHP_URL_SCHEME 0
#define PTN_PHP_URL_HOST 1
#define PTN_PHP_URL_PORT 2
#define PTN_PHP_URL_USER 3
#define PTN_PHP_URL_PASS 4
#define PTN_PHP_URL_PATH 5
#define PTN_PHP_URL_QUERY 6
#define PTN_PHP_URL_FRAGMENT 7
#define PTN_PHP_QUERY_RFC1738 1
#define PTN_PHP_QUERY_RFC3986 2
#define PTN_FILE_USE_INCLUDE_PATH 1
#define PTN_FILE_IGNORE_NEW_LINES 2
#define PTN_FILE_SKIP_EMPTY_LINES 4
#define PTN_FILE_APPEND 8
#define PTN_FILE_NO_DEFAULT_CONTEXT 16
#define PTN_SCANDIR_SORT_ASCENDING 0
#define PTN_SCANDIR_SORT_DESCENDING 1
#define PTN_SCANDIR_SORT_NONE 2
#define PTN_LOCK_SH 1
#define PTN_LOCK_EX 2
#define PTN_LOCK_UN 3
#define PTN_LOCK_NB 4
#define PTN_FNM_NOESCAPE 1
#define PTN_FNM_PATHNAME 2
#define PTN_FNM_PERIOD 4
#define PTN_FNM_CASEFOLD 16
#define PTN_GLOB_MARK 2
#define PTN_GLOB_NOSORT 4
#define PTN_GLOB_NOCHECK 16
#define PTN_GLOB_NOESCAPE 64
#define PTN_GLOB_BRACE 1024
#define PTN_GLOB_ONLYDIR 8192
#define PTN_GLOB_ERR 1
#define PTN_SEEK_SET SEEK_SET
#define PTN_SEEK_CUR SEEK_CUR
#define PTN_SEEK_END SEEK_END
#define PTN_STREAM_FILTER_READ 1
#define PTN_STREAM_FILTER_WRITE 2
#define PTN_STREAM_FILTER_ALL 3
#define PTN_STREAM_IS_URL 1
#define PTN_STREAM_REPORT_ERRORS 8
#define PTN_DNS_A 1
#define PTN_DNS_NS 2
#define PTN_DNS_CNAME 16
#define PTN_DNS_SOA 32
#define PTN_DNS_PTR 2048
#define PTN_DNS_HINFO 4096
#define PTN_DNS_CAA 8192
#define PTN_DNS_MX 16384
#define PTN_DNS_TXT 32768
#define PTN_DNS_A6 16777216
#define PTN_DNS_SRV 33554432
#define PTN_DNS_NAPTR 67108864
#define PTN_DNS_AAAA 134217728
#define PTN_DNS_ALL 251721779
#define PTN_DNS_ANY 268435456
#define PTN_PSFS_ERR_FATAL 0
#define PTN_PSFS_FEED_ME 1
#define PTN_PSFS_PASS_ON 2
#define PTN_STREAM_OOB 1
#define PTN_STREAM_PEEK 2
#define PTN_STREAM_CLIENT_PERSISTENT 1
#define PTN_STREAM_CLIENT_ASYNC_CONNECT 2
#define PTN_STREAM_CLIENT_CONNECT 4
#define PTN_STREAM_SERVER_BIND 4
#define PTN_STREAM_SERVER_LISTEN 8
#define PTN_ZLIB_ENCODING_RAW -15
#define PTN_ZLIB_ENCODING_GZIP 31
#define PTN_ZLIB_ENCODING_DEFLATE 15
#define PTN_ZLIB_VERSION "1.3.1"
#define PTN_ZLIB_VERNUM 0x1310
#define PTN_FORCE_GZIP 31
#define PTN_FORCE_DEFLATE 15
#define PTN_ZLIB_OK 0
#define PTN_ZLIB_STREAM_END 1
#define PTN_ZLIB_NO_FLUSH 0
#define PTN_ZLIB_PARTIAL_FLUSH 1
#define PTN_ZLIB_SYNC_FLUSH 2
#define PTN_ZLIB_FULL_FLUSH 3
#define PTN_ZLIB_BLOCK 5
#define PTN_ZLIB_FINISH 4
#if defined(AF_UNIX)
#define PTN_STREAM_PF_UNIX AF_UNIX
#else
#define PTN_STREAM_PF_UNIX 1
#endif
#if defined(SOCK_STREAM)
#define PTN_STREAM_SOCK_STREAM SOCK_STREAM
#else
#define PTN_STREAM_SOCK_STREAM 1
#endif
#if defined(IPPROTO_IP)
#define PTN_STREAM_IPPROTO_IP IPPROTO_IP
#else
#define PTN_STREAM_IPPROTO_IP 0
#endif
#if defined(SHUT_RD)
#define PTN_STREAM_SHUT_RD SHUT_RD
#else
#define PTN_STREAM_SHUT_RD 0
#endif
#if defined(SHUT_WR)
#define PTN_STREAM_SHUT_WR SHUT_WR
#else
#define PTN_STREAM_SHUT_WR 1
#endif
#if defined(SHUT_RDWR)
#define PTN_STREAM_SHUT_RDWR SHUT_RDWR
#else
#define PTN_STREAM_SHUT_RDWR 2
#endif
#define PTN_DEBUG_BACKTRACE_PROVIDE_OBJECT 1
#define PTN_DEBUG_BACKTRACE_IGNORE_ARGS 2
#define PTN_LC_CTYPE 0
#define PTN_LC_NUMERIC 1
#define PTN_LC_TIME 2
#define PTN_LC_COLLATE 3
#define PTN_LC_MONETARY 4
#define PTN_LC_MESSAGES 5
#define PTN_LC_ALL 6
#define PTN_ABDAY_1 131072
#define PTN_ABDAY_2 131073
#define PTN_ABDAY_3 131074
#define PTN_ABDAY_4 131075
#define PTN_ABDAY_5 131076
#define PTN_ABDAY_6 131077
#define PTN_ABDAY_7 131078
#define PTN_DAY_1 131079
#define PTN_DAY_2 131080
#define PTN_DAY_3 131081
#define PTN_DAY_4 131082
#define PTN_DAY_5 131083
#define PTN_DAY_6 131084
#define PTN_DAY_7 131085
#define PTN_ABMON_1 131086
#define PTN_ABMON_2 131087
#define PTN_ABMON_3 131088
#define PTN_ABMON_4 131089
#define PTN_ABMON_5 131090
#define PTN_ABMON_6 131091
#define PTN_ABMON_7 131092
#define PTN_ABMON_8 131093
#define PTN_ABMON_9 131094
#define PTN_ABMON_10 131095
#define PTN_ABMON_11 131096
#define PTN_ABMON_12 131097
#define PTN_MON_1 131098
#define PTN_MON_2 131099
#define PTN_MON_3 131100
#define PTN_MON_4 131101
#define PTN_MON_5 131102
#define PTN_MON_6 131103
#define PTN_MON_7 131104
#define PTN_MON_8 131105
#define PTN_MON_9 131106
#define PTN_MON_10 131107
#define PTN_MON_11 131108
#define PTN_MON_12 131109
#define PTN_RADIXCHAR 65536
#define PTN_THOUSEP 65537
#define PTN_YESEXPR 327680
#define PTN_NOEXPR 327681
#define PTN_CODESET 14
#define PTN_DEFAULT_PRECISION 14
#define PTN_DEFAULT_SERIALIZE_PRECISION -1
#define PTN_MAX_FLOAT_FORMAT_PRECISION 1000
#define PTN_FLOAT_FORMAT_BUFFER_SIZE 1200
#define PTN_JSON_ERROR_NONE 0
#define PTN_JSON_ERROR_DEPTH 1
#define PTN_JSON_ERROR_STATE_MISMATCH 2
#define PTN_JSON_ERROR_CTRL_CHAR 3
#define PTN_JSON_ERROR_SYNTAX 4
#define PTN_JSON_ERROR_UTF8 5
#define PTN_JSON_ERROR_RECURSION 6
#define PTN_JSON_ERROR_INF_OR_NAN 7
#define PTN_JSON_ERROR_UNSUPPORTED_TYPE 8
#define PTN_JSON_ERROR_INVALID_PROPERTY_NAME 9
#define PTN_JSON_ERROR_UTF16 10
#define PTN_JSON_ERROR_NON_BACKED_ENUM 11
#define PTN_JSON_OBJECT_AS_ARRAY 1
#define PTN_JSON_BIGINT_AS_STRING 2
#define PTN_JSON_HEX_TAG 1
#define PTN_JSON_HEX_AMP 2
#define PTN_JSON_HEX_APOS 4
#define PTN_JSON_HEX_QUOT 8
#define PTN_JSON_FORCE_OBJECT 16
#define PTN_JSON_NUMERIC_CHECK 32
#define PTN_JSON_UNESCAPED_SLASHES 64
#define PTN_JSON_PRETTY_PRINT 128
#define PTN_JSON_UNESCAPED_UNICODE 256
#define PTN_JSON_PARTIAL_OUTPUT_ON_ERROR 512
#define PTN_JSON_PRESERVE_ZERO_FRACTION 1024
#define PTN_JSON_UNESCAPED_LINE_TERMINATORS 2048
#define PTN_JSON_INVALID_UTF8_IGNORE 1048576
#define PTN_JSON_INVALID_UTF8_SUBSTITUTE 2097152
#define PTN_JSON_THROW_ON_ERROR 4194304
#define PTN_LAZY_OBJECT_SKIP_INITIALIZATION_ON_SERIALIZE 8
#define PTN_LAZY_OBJECT_SKIP_DESTRUCTOR 16
#define PTN_LAZY_OBJECT_USER_MASK \
    (PTN_LAZY_OBJECT_SKIP_INITIALIZATION_ON_SERIALIZE | PTN_LAZY_OBJECT_SKIP_DESTRUCTOR)
#define PTN_PREG_PATTERN_ORDER 1
#define PTN_PREG_SET_ORDER 2
#define PTN_PREG_OFFSET_CAPTURE 256
#define PTN_PREG_UNMATCHED_AS_NULL 512
#define PTN_PREG_SPLIT_NO_EMPTY 1
#define PTN_PREG_SPLIT_DELIM_CAPTURE 2
#define PTN_PREG_SPLIT_OFFSET_CAPTURE 4
#define PTN_PREG_GREP_INVERT 1
#define PTN_PREG_NO_ERROR 0
#define PTN_PREG_INTERNAL_ERROR 1
#define PTN_PREG_BACKTRACK_LIMIT_ERROR 2
#define PTN_PREG_RECURSION_LIMIT_ERROR 3
#define PTN_PREG_BAD_UTF8_ERROR 4
#define PTN_PREG_BAD_UTF8_OFFSET_ERROR 5
#define PTN_PREG_JIT_STACKLIMIT_ERROR 6
#define PTN_FILTER_VALIDATE_INT 257
#define PTN_FILTER_VALIDATE_BOOLEAN 258
#define PTN_FILTER_VALIDATE_FLOAT 259
#define PTN_FILTER_VALIDATE_REGEXP 272
#define PTN_FILTER_VALIDATE_DOMAIN 277
#define PTN_FILTER_VALIDATE_URL 273
#define PTN_FILTER_VALIDATE_EMAIL 274
#define PTN_FILTER_VALIDATE_IP 275
#define PTN_FILTER_VALIDATE_MAC 276
#define PTN_FILTER_SANITIZE_STRING 513
#define PTN_FILTER_SANITIZE_STRIPPED 513
#define PTN_FILTER_SANITIZE_ENCODED 514
#define PTN_FILTER_SANITIZE_SPECIAL_CHARS 515
#define PTN_FILTER_UNSAFE_RAW 516
#define PTN_FILTER_DEFAULT PTN_FILTER_UNSAFE_RAW
#define PTN_FILTER_SANITIZE_EMAIL 517
#define PTN_FILTER_SANITIZE_URL 518
#define PTN_FILTER_SANITIZE_NUMBER_INT 519
#define PTN_FILTER_SANITIZE_NUMBER_FLOAT 520
#define PTN_FILTER_SANITIZE_FULL_SPECIAL_CHARS 522
#define PTN_FILTER_SANITIZE_ADD_SLASHES 523
#define PTN_FILTER_CALLBACK 1024
#define PTN_FILTER_FLAG_ALLOW_OCTAL 1
#define PTN_FILTER_FLAG_ALLOW_HEX 2
#define PTN_FILTER_FLAG_STRIP_LOW 4
#define PTN_FILTER_FLAG_STRIP_HIGH 8
#define PTN_FILTER_FLAG_ENCODE_LOW 16
#define PTN_FILTER_FLAG_ENCODE_HIGH 32
#define PTN_FILTER_FLAG_ENCODE_AMP 64
#define PTN_FILTER_FLAG_NO_ENCODE_QUOTES 128
#define PTN_FILTER_FLAG_EMPTY_STRING_NULL 256
#define PTN_FILTER_FLAG_STRIP_BACKTICK 512
#define PTN_FILTER_FLAG_ALLOW_FRACTION 4096
#define PTN_FILTER_FLAG_ALLOW_THOUSAND 8192
#define PTN_FILTER_FLAG_ALLOW_SCIENTIFIC 16384
#define PTN_FILTER_FLAG_PATH_REQUIRED 262144
#define PTN_FILTER_FLAG_QUERY_REQUIRED 524288
#define PTN_FILTER_FLAG_IPV4 1048576
#define PTN_FILTER_FLAG_HOSTNAME 1048576
#define PTN_FILTER_FLAG_EMAIL_UNICODE 1048576
#define PTN_FILTER_FLAG_IPV6 2097152
#define PTN_FILTER_FLAG_NO_RES_RANGE 4194304
#define PTN_FILTER_FLAG_NO_PRIV_RANGE 8388608
#define PTN_FILTER_FLAG_GLOBAL_RANGE 268435456
#define PTN_FILTER_THROW_ON_FAILURE 268435456
#define PTN_FILTER_REQUIRE_ARRAY 16777216
#define PTN_FILTER_REQUIRE_SCALAR 33554432
#define PTN_FILTER_FORCE_ARRAY 67108864
#define PTN_FILTER_NULL_ON_FAILURE 134217728
#define PTN_INPUT_POST 0
#define PTN_INPUT_GET 1
#define PTN_INPUT_COOKIE 2
#define PTN_INPUT_ENV 4
#define PTN_INPUT_SERVER 5
#define PTN_MB_CASE_UPPER 0
#define PTN_MB_CASE_LOWER 1
#define PTN_MB_CASE_TITLE 2
#define PTN_MB_CASE_FOLD 3
#define PTN_MB_CASE_UPPER_SIMPLE 4
#define PTN_MB_CASE_LOWER_SIMPLE 5
#define PTN_MB_CASE_TITLE_SIMPLE 6
#define PTN_MB_CASE_FOLD_SIMPLE 7
#define PTN_MB_ONIGURUMA_VERSION "6.9.10"
#define PTN_HASH_HMAC 1
#define PTN_INTL_ICU_VERSION "73.2"
#define PTN_GRAPHEME_EXTR_COUNT 0
#define PTN_GRAPHEME_EXTR_MAXBYTES 1
#define PTN_GRAPHEME_EXTR_MAXCHARS 2
#define PTN_INTL_BREAK_ITERATOR_DONE -1
#define PTN_INTL_PARTS_KEY_SEQUENTIAL 0
#define PTN_INTL_PARTS_KEY_LEFT 1
#define PTN_INTL_PARTS_KEY_RIGHT 2
#define PTN_NUMBER_FORMATTER_PATTERN_DECIMAL 0
#define PTN_NUMBER_FORMATTER_DECIMAL 1
#define PTN_NUMBER_FORMATTER_CURRENCY 2
#define PTN_NUMBER_FORMATTER_PERCENT 3
#define PTN_NUMBER_FORMATTER_SCIENTIFIC 4
#define PTN_NUMBER_FORMATTER_SPELLOUT 5
#define PTN_NUMBER_FORMATTER_ORDINAL 6
#define PTN_NUMBER_FORMATTER_DURATION 7
#define PTN_NUMBER_FORMATTER_PATTERN_RULEBASED 9
#define PTN_NUMBER_FORMATTER_CURRENCY_ACCOUNTING 12
#define PTN_NUMBER_FORMATTER_DECIMAL_COMPACT_SHORT 14
#define PTN_NUMBER_FORMATTER_DECIMAL_COMPACT_LONG 15
#define PTN_NUMBER_FORMATTER_TYPE_DEFAULT 0
#define PTN_NUMBER_FORMATTER_TYPE_INT32 1
#define PTN_NUMBER_FORMATTER_TYPE_INT64 2
#define PTN_NUMBER_FORMATTER_TYPE_DOUBLE 3
#define PTN_NUMBER_FORMATTER_TYPE_CURRENCY 4
#define PTN_NUMBER_FORMATTER_DECIMAL_SEPARATOR_SYMBOL 0
#define PTN_NUMBER_FORMATTER_GROUPING_SEPARATOR_SYMBOL 1
#define PTN_NUMBER_FORMATTER_PATTERN_SEPARATOR_SYMBOL 2
#define PTN_NUMBER_FORMATTER_PERCENT_SYMBOL 3
#define PTN_NUMBER_FORMATTER_ZERO_DIGIT_SYMBOL 4
#define PTN_NUMBER_FORMATTER_DIGIT_SYMBOL 5
#define PTN_NUMBER_FORMATTER_MINUS_SIGN_SYMBOL 6
#define PTN_NUMBER_FORMATTER_PLUS_SIGN_SYMBOL 7
#define PTN_NUMBER_FORMATTER_CURRENCY_SYMBOL 8
#define PTN_NUMBER_FORMATTER_INTL_CURRENCY_SYMBOL 9
#define PTN_NUMBER_FORMATTER_MONETARY_SEPARATOR_SYMBOL 10
#define PTN_NUMBER_FORMATTER_EXPONENTIAL_SYMBOL 11
#define PTN_NUMBER_FORMATTER_PERMILL_SYMBOL 12
#define PTN_NUMBER_FORMATTER_PAD_ESCAPE_SYMBOL 13
#define PTN_NUMBER_FORMATTER_INFINITY_SYMBOL 14
#define PTN_NUMBER_FORMATTER_NAN_SYMBOL 15
#define PTN_NUMBER_FORMATTER_SIGNIFICANT_DIGIT_SYMBOL 16
#define PTN_NUMBER_FORMATTER_MONETARY_GROUPING_SEPARATOR_SYMBOL 17
#define PTN_NUMBER_FORMATTER_SYMBOL_COUNT 18
#define PTN_INTL_NUMBER_RANGE_COLLAPSE_AUTO 0
#define PTN_INTL_NUMBER_RANGE_COLLAPSE_NONE 1
#define PTN_INTL_NUMBER_RANGE_COLLAPSE_UNIT 2
#define PTN_INTL_NUMBER_RANGE_COLLAPSE_ALL 3
#define PTN_INTL_NUMBER_RANGE_IDENTITY_FALLBACK_SINGLE_VALUE 0
#define PTN_INTL_NUMBER_RANGE_IDENTITY_FALLBACK_APPROXIMATELY_OR_SINGLE_VALUE 1
#define PTN_INTL_NUMBER_RANGE_IDENTITY_FALLBACK_APPROXIMATELY 2
#define PTN_INTL_NUMBER_RANGE_IDENTITY_FALLBACK_RANGE 3
#define PTN_TOKEN_PARSE 1
#define PTN_T_INCLUDE 1000
#define PTN_T_INCLUDE_ONCE 1001
#define PTN_T_EVAL 1002
#define PTN_T_REQUIRE 1003
#define PTN_T_REQUIRE_ONCE 1004
#define PTN_T_LOGICAL_OR 1005
#define PTN_T_LOGICAL_XOR 1006
#define PTN_T_LOGICAL_AND 1007
#define PTN_T_PRINT 1008
#define PTN_T_PLUS_EQUAL 1009
#define PTN_T_MINUS_EQUAL 1010
#define PTN_T_MUL_EQUAL 1011
#define PTN_T_DIV_EQUAL 1012
#define PTN_T_CONCAT_EQUAL 1013
#define PTN_T_MOD_EQUAL 1014
#define PTN_T_AND_EQUAL 1015
#define PTN_T_OR_EQUAL 1016
#define PTN_T_XOR_EQUAL 1017
#define PTN_T_SL_EQUAL 1018
#define PTN_T_SR_EQUAL 1019
#define PTN_T_BOOLEAN_OR 1020
#define PTN_T_BOOLEAN_AND 1021
#define PTN_T_IS_EQUAL 1022
#define PTN_T_IS_NOT_EQUAL 1023
#define PTN_T_IS_IDENTICAL 1024
#define PTN_T_IS_NOT_IDENTICAL 1025
#define PTN_T_IS_SMALLER_OR_EQUAL 1026
#define PTN_T_IS_GREATER_OR_EQUAL 1027
#define PTN_T_SL 1028
#define PTN_T_SR 1029
#define PTN_T_INC 1030
#define PTN_T_DEC 1031
#define PTN_T_INT_CAST 1032
#define PTN_T_DOUBLE_CAST 1033
#define PTN_T_STRING_CAST 1034
#define PTN_T_ARRAY_CAST 1035
#define PTN_T_OBJECT_CAST 1036
#define PTN_T_BOOL_CAST 1037
#define PTN_T_UNSET_CAST 1038
#define PTN_T_NEW 1039
#define PTN_T_EXIT 1040
#define PTN_T_IF 1041
#define PTN_T_ELSEIF 1042
#define PTN_T_ELSE 1043
#define PTN_T_ENDIF 1044
#define PTN_T_LNUMBER 1045
#define PTN_T_DNUMBER 1046
#define PTN_T_STRING 1047
#define PTN_T_STRING_VARNAME 1048
#define PTN_T_VARIABLE 1049
#define PTN_T_NUM_STRING 1050
#define PTN_T_INLINE_HTML 1051
#define PTN_T_ENCAPSED_AND_WHITESPACE 1052
#define PTN_T_CONSTANT_ENCAPSED_STRING 1053
#define PTN_T_ECHO 1054
#define PTN_T_DO 1055
#define PTN_T_WHILE 1056
#define PTN_T_ENDWHILE 1057
#define PTN_T_FOR 1058
#define PTN_T_ENDFOR 1059
#define PTN_T_FOREACH 1060
#define PTN_T_ENDFOREACH 1061
#define PTN_T_DECLARE 1062
#define PTN_T_ENDDECLARE 1063
#define PTN_T_AS 1064
#define PTN_T_SWITCH 1065
#define PTN_T_ENDSWITCH 1066
#define PTN_T_CASE 1067
#define PTN_T_DEFAULT 1068
#define PTN_T_BREAK 1069
#define PTN_T_CONTINUE 1070
#define PTN_T_FUNCTION 1071
#define PTN_T_CONST 1072
#define PTN_T_RETURN 1073
#define PTN_T_USE 1074
#define PTN_T_GLOBAL 1075
#define PTN_T_STATIC 1076
#define PTN_T_VAR 1077
#define PTN_T_UNSET 1078
#define PTN_T_ISSET 1079
#define PTN_T_EMPTY 1080
#define PTN_T_CLASS 1081
#define PTN_T_EXTENDS 1082
#define PTN_T_INTERFACE 1083
#define PTN_T_IMPLEMENTS 1084
#define PTN_T_OBJECT_OPERATOR 1085
#define PTN_T_DOUBLE_ARROW 1086
#define PTN_T_LIST 1087
#define PTN_T_ARRAY 1088
#define PTN_T_CLASS_C 1089
#define PTN_T_FUNC_C 1090
#define PTN_T_PROPERTY_C 1091
#define PTN_T_METHOD_C 1092
#define PTN_T_LINE 1093
#define PTN_T_FILE 1094
#define PTN_T_COMMENT 1095
#define PTN_T_DOC_COMMENT 1096
#define PTN_T_OPEN_TAG 1097
#define PTN_T_OPEN_TAG_WITH_ECHO 1098
#define PTN_T_CLOSE_TAG 1099
#define PTN_T_WHITESPACE 1100
#define PTN_T_START_HEREDOC 1101
#define PTN_T_END_HEREDOC 1102
#define PTN_T_DOLLAR_OPEN_CURLY_BRACES 1103
#define PTN_T_CURLY_OPEN 1104
#define PTN_T_DOUBLE_COLON 1105
#define PTN_T_PAAMAYIM_NEKUDOTAYIM PTN_T_DOUBLE_COLON
#define PTN_T_ABSTRACT 1106
#define PTN_T_CATCH 1107
#define PTN_T_FINAL 1108
#define PTN_T_INSTANCEOF 1109
#define PTN_T_PRIVATE 1110
#define PTN_T_PROTECTED 1111
#define PTN_T_PUBLIC 1112
#define PTN_T_THROW 1113
#define PTN_T_TRY 1114
#define PTN_T_CLONE 1115
#define PTN_T_HALT_COMPILER 1116
#define PTN_T_NAME_FULLY_QUALIFIED 1117
#define PTN_T_NAME_RELATIVE 1118
#define PTN_T_NAME_QUALIFIED 1119
#define PTN_T_NS_SEPARATOR 1120
#define PTN_T_NULLSAFE_OBJECT_OPERATOR 1121
#define PTN_T_ATTRIBUTE 1122
#define PTN_T_BAD_CHARACTER 1123
#define PTN_T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG 1124
#define PTN_T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG 1125
#define PTN_TOKEN_PARSE 1

typedef struct PtnArray PtnArray;
typedef struct PtnClosure PtnClosure;
typedef struct PtnException PtnException;
typedef struct PtnGenerator PtnGenerator;
typedef struct PtnObject PtnObject;
typedef struct PtnReference PtnReference;
typedef struct PtnRuntime PtnRuntime;
typedef struct PtnResource PtnResource;
typedef struct PtnStreamFilter PtnStreamFilter;
typedef struct PtnTraceFrame PtnTraceFrame;
typedef struct PtnTryFrame PtnTryFrame;

static int ptn_builtin_class_implements_interface(const char *class_name, const char *interface_name);

typedef enum {
    PTN_NULL,
    PTN_BOOL,
    PTN_INT,
    PTN_FLOAT,
    PTN_STRING,
    PTN_RESOURCE,
    PTN_ARRAY,
    PTN_OBJECT,
    PTN_CLOSURE,
    PTN_EXCEPTION,
    PTN_REFERENCE
} PtnType;

typedef enum {
    PTN_ARRAY_KEY_INT,
    PTN_ARRAY_KEY_STRING
} PtnArrayKeyType;

typedef enum {
    PTN_STREAM_FILTER_STRING_ROT13,
    PTN_STREAM_FILTER_STRING_TOUPPER,
    PTN_STREAM_FILTER_STRING_TOLOWER,
    PTN_STREAM_FILTER_CONVERT_BASE64_ENCODE,
    PTN_STREAM_FILTER_CONVERT_BASE64_DECODE,
    PTN_STREAM_FILTER_CONVERT_QUOTED_PRINTABLE_ENCODE,
    PTN_STREAM_FILTER_CONVERT_QUOTED_PRINTABLE_DECODE,
    PTN_STREAM_FILTER_CONVERT_ICONV,
    PTN_STREAM_FILTER_DECHUNK,
    PTN_STREAM_FILTER_ZLIB_DEFLATE,
    PTN_STREAM_FILTER_ZLIB_INFLATE,
    PTN_STREAM_FILTER_USER
} PtnStreamFilterKind;

typedef struct {
    size_t refcount;
    size_t len;
    unsigned char *data;
    int interned;
} PtnStringPayload;

typedef struct {
    const unsigned char *data;
    size_t len;
    PtnStringPayload *payload;
} PtnString;

typedef struct {
    PtnArrayKeyType type;
    size_t string_len;
    union {
        int64_t integer;
        const char *string;
    } as;
} PtnArrayKey;

typedef struct {
    PtnType type;
    int owned;
    int by_ref_return_fallback;
    int by_ref_argument_source_disabled;
    int from_string_offset;
    union {
        int boolean;
        int64_t integer;
        double floating;
        PtnString string;
        PtnArray *array;
        PtnObject *object;
        PtnClosure *closure;
        PtnException *exception;
        PtnResource *resource;
        PtnReference *reference;
    } as;
} PtnValue;

static PTN_UNUSED PtnValue ptn_value_with_by_ref_return_fallback(PtnValue value, int fallback) {
    value.by_ref_return_fallback = fallback ? 1 : 0;
    return value;
}

static PTN_UNUSED PtnValue ptn_value_from_string_offset(PtnValue value) {
    value.from_string_offset = 1;
    return value;
}

typedef PtnValue (*PtnFunctionStaticVariablesProvider)(PtnRuntime *runtime);

typedef struct {
    char *name;
    size_t name_len;
    PtnValue value;
} PtnSymbol;

typedef struct {
    int occupied;
    uint64_t hash;
    size_t symbol_index;
} PtnSymbolIndexSlot;

typedef struct {
    PtnSymbol *items;
    size_t len;
    size_t capacity;
    PtnSymbolIndexSlot *index_slots;
    size_t index_capacity;
    uint64_t mutation_epoch;
} PtnSymbolTable;

typedef PtnValue (*PtnParameterDefaultProvider)(
    PtnRuntime *runtime,
    const char *scope_class_name,
    size_t line
);

typedef struct {
    const char *name;
    const char *type_name;
    const char *type_display_name;
    int type_allows_null;
    int type_is_builtin;
    int by_ref;
    int is_variadic;
    int can_be_passed_by_value;
    const char *default_value_display;
    const char *default_value_constant_name;
    const char *doc_comment;
    PtnParameterDefaultProvider default_value_provider;
} PtnParameterMetadata;

typedef struct {
    int found;
    const char *name;
    int is_internal;
    size_t parameter_count;
    size_t required_parameter_count;
    int is_variadic;
    const PtnParameterMetadata *parameters;
    int return_by_ref;
    int is_generator;
    int is_deprecated;
    const char *return_type_name;
    const char *return_type_display_name;
    int return_type_allows_null;
    int return_type_is_builtin;
    const char *tentative_return_type_name;
    const char *tentative_return_type_display_name;
    int tentative_return_type_allows_null;
    int tentative_return_type_is_builtin;
    const char *source_file;
    size_t start_line;
    size_t end_line;
    const char *doc_comment;
    PtnFunctionStaticVariablesProvider static_variables_provider;
    int has_user_function_index;
    size_t user_function_index;
    const char *attribute_method_name;
} PtnFunctionMetadata;

enum {
    PTN_CLOSURE_ORIGIN_ANONYMOUS = 0,
    PTN_CLOSURE_ORIGIN_FUNCTION = 1,
    PTN_CLOSURE_ORIGIN_STATIC_METHOD = 2,
    PTN_CLOSURE_ORIGIN_METHOD = 3
};

struct PtnClosure {
    size_t refcount;
    size_t object_id;
    size_t gc_mark_epoch;
    PtnRuntime *lifecycle_runtime;
    size_t function_index;
    const char *display_name;
    PtnFunctionMetadata metadata;
    char *scope_class_name;
    char *called_class_name;
    int is_static;
    int uses_this;
    PtnSymbolTable captures;
    int has_wrapped_callable;
    int suppress_wrapped_callable_deprecation;
    PtnValue wrapped_callable;
    char *bound_scope_name;
    int origin_kind;
    char *origin_class_name;
    char *origin_method_name;
};

typedef enum {
    PTN_PROPERTY_PUBLIC,
    PTN_PROPERTY_PROTECTED,
    PTN_PROPERTY_PRIVATE
} PtnPropertyVisibility;

typedef enum {
    PTN_PROPERTY_TYPE_NONE,
    PTN_PROPERTY_TYPE_NULL,
    PTN_PROPERTY_TYPE_ARRAY,
    PTN_PROPERTY_TYPE_INT,
    PTN_PROPERTY_TYPE_FLOAT,
    PTN_PROPERTY_TYPE_STRING,
    PTN_PROPERTY_TYPE_BOOL,
    PTN_PROPERTY_TYPE_MIXED,
    PTN_PROPERTY_TYPE_OBJECT,
    PTN_PROPERTY_TYPE_CLASS,
    PTN_PROPERTY_TYPE_TEXT
} PtnPropertyTypeKind;

typedef struct {
    PtnPropertyTypeKind kind;
    char *class_name;
    char *text;
    int allows_null;
    char *declaring_class;
    char *property_name;
} PtnReferencePropertyTypeSource;

struct PtnReference {
    size_t refcount;
    PtnValue value;
    PtnRuntime *lifecycle_runtime;
    size_t live_index;
    size_t gc_mark_epoch;
    int gc_collecting;
    PtnPropertyTypeKind property_type_kind;
    char *property_type_class_name;
    char *property_type_text;
    int property_type_allows_null;
    char *property_declaring_class;
    char *property_name;
    PtnReferencePropertyTypeSource *property_type_sources;
    size_t property_type_source_len;
    size_t property_type_source_cap;
};

typedef struct {
    char *storage_name;
    char *display_name;
    char *declaring_class;
    PtnPropertyVisibility read_visibility;
    PtnPropertyVisibility set_visibility;
    int is_readonly;
    int has_hooks;
    int is_virtual;
    int hook_has_get;
    int hook_get_returns_by_ref;
    int hook_has_set;
    char *hook_get_declaring_class;
    char *hook_set_declaring_class;
    int is_unset;
    int lazy_skip;
    int readonly_clone_reinitialized;
    char *last_type_name;
    PtnPropertyTypeKind type_kind;
    char *type_class_name;
    char *type_text;
    int type_allows_null;
} PtnObjectPropertyMetadata;

typedef void (*PtnObjectNativeDataFree)(void *data);

typedef struct {
    int exists;
    PtnValue value;
} PtnLookupResult;

static PTN_UNUSED int ptn_property_reference_coerce_assignment(
    PtnRuntime *runtime,
    const PtnReference *reference,
    PtnValue value,
    int reference_context,
    size_t line,
    PtnValue *out
);
static PTN_UNUSED int ptn_property_type_coerce_assignment(
    PtnRuntime *runtime,
    PtnPropertyTypeKind kind,
    const char *type_class_name,
    const char *type_text,
    int allows_null,
    const char *declaring_class,
    const char *property,
    PtnValue value,
    int reference_context,
    size_t line,
    PtnValue *out
);
static PTN_UNUSED int ptn_property_type_try_coerce_assignment(
    PtnRuntime *runtime,
    PtnPropertyTypeKind kind,
    const char *type_class_name,
    const char *type_text,
    int allows_null,
    PtnValue value,
    PtnValue *out
);
static PTN_UNUSED void ptn_reference_adopt_property_type(
    PtnReference *reference,
    const PtnObjectPropertyMetadata *metadata
);
static PTN_UNUSED void ptn_throw_reference_property_bind_incompatibility(
    PtnRuntime *runtime,
    PtnValue value,
    const PtnReferencePropertyTypeSource *existing,
    const PtnObjectPropertyMetadata *metadata
);
static PTN_UNUSED PtnReferencePropertyTypeSource ptn_reference_primary_property_type_source(
    const PtnReference *reference
);
static PTN_UNUSED int ptn_compare_identical(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line
);
static PTN_UNUSED PtnValue ptn_cast_string(PtnValue value);
static PTN_UNUSED char *ptn_value_to_string(PtnValue value);
static PTN_UNUSED int ptn_value_satisfies_class_type_hint(PtnRuntime *runtime, PtnValue value, const char *expected_class_name);

typedef struct {
    int append;
    PtnValue value;
    const char *deferred_missing_variable_name;
    size_t deferred_missing_variable_line;
} PtnArrayPathSegment;

typedef struct {
    PtnArrayKey key;
    PtnValue value;
    int by_ref_argument_eligible;
} PtnArrayEntry;

typedef struct {
    int occupied;
    uint64_t hash;
    size_t entry_index;
} PtnArrayIndexSlot;

typedef struct {
    char *data;
    size_t len;
    size_t capacity;
} PtnStringBuffer;

typedef struct {
    PtnArray *array;
    PtnObject *object;
    PtnGenerator *generator;
    PtnRuntime *runtime;
    const char *access_scope;
    PtnValue iterator_object;
    size_t index;
    size_t length;
    PtnArrayKey current_key;
    PtnReference *current_reference;
    PtnValue *watched_slot;
    size_t line;
    uint64_t seen_mutation_epoch;
    int has_current_key;
    int has_iterator_object;
    int protocol_iterator;
    int spl_dllist_delete;
    int spl_dllist_reverse;
    int object_property_iterator;
    int valid;
    int live;
} PtnArrayIterator;

struct PtnGenerator {
    PtnArray *values;
    PtnArray *keys;
    PtnObject *object;
    PtnValue return_value;
    PtnArray *reference_notice_lines;
    PtnArray *yield_lines;
    PtnArray *delegate_sources;
    PtnArray *force_close_yield_from_entries;
    PtnArray *output_chunks;
    PtnArray *send_call_positions;
    PtnArray *send_call_kinds;
    PtnArray *send_call_names;
    PtnArray *send_call_receivers;
    PtnArray *send_call_arguments;
    PtnArray *send_call_yield_indexes;
    PtnArray *send_call_lines;
    PtnArray *send_yield_from_positions;
    PtnArray *send_yield_from_lines;
    PtnValue pending_exception;
    size_t pending_exception_position;
    int has_pending_exception;
    int pending_exception_on_rewind;
    size_t return_yield_position;
    int has_return_yield_position;
    PtnStringBuffer pending_output;
    PtnValue closure_owner;
    int has_receiver;
    PtnValue receiver;
    char *function_name;
    char *source_file;
    size_t source_line;
    size_t position;
    int64_t next_auto_key;
    int completed;
    int started;
    int executing;
    int force_closing;
    int yields_by_ref;
};

struct PtnArray {
    size_t refcount;
    int destructing;
    size_t gc_mark_epoch;
    PtnRuntime *lifecycle_runtime;
    size_t live_index;
    size_t debug_hidden_refcount;
    int debug_reference_wrapped;
    size_t iterator_refcount;
    size_t len;
    size_t capacity;
    PtnArrayEntry *entries;
    PtnArrayIndexSlot *index_slots;
    size_t index_capacity;
    int64_t next_auto_key;
    size_t current_index;
    int has_iterator_current_index;
    size_t iterator_current_index;
    size_t iterator_mutation_resume_index;
    uint64_t iterator_mutation_epoch;
    uint64_t mutation_epoch;
};

struct PtnObject {
    size_t refcount;
    size_t debug_hidden_refcount;
    size_t object_id;
    size_t gc_mark_epoch;
    char *class_name;
    char *enum_case_name;
    PtnArray *properties;
    PtnObjectPropertyMetadata *property_metadata;
    size_t property_metadata_len;
    size_t property_metadata_capacity;
    void *native_data;
    PtnObjectNativeDataFree native_data_free;
    PtnRuntime *lifecycle_runtime;
    size_t live_index;
    int destructor_enabled;
    int destructor_called;
    int lazy_uninitialized;
    int lazy_is_proxy;
    int lazy_options;
    int lazy_initializing;
    size_t lazy_initializer_refcount_guards;
    int readonly_clone_initializing;
    int defer_object_id_release_once;
    int var_dump_property_count_initialized;
    size_t last_var_dump_property_count;
    size_t active_property_value_unsets;
    PtnValue lazy_initializer;
    PtnValue lazy_proxy_instance;
};

typedef struct {
    int has_key;
    PtnValue key;
    PtnValue value;
} PtnArrayLiteralEntry;

typedef struct {
    PtnStringBuffer buffer;
    int has_callback;
    PtnValue callback;
    size_t chunk_size;
    int64_t flags;
    int trans_sid_rewrite;
    char *trans_sid_session_name;
    char *trans_sid_session_id;
    char *trans_sid_hosts;
} PtnOutputBuffer;

typedef struct {
    PtnValue callback;
    PtnValue *args;
    size_t argc;
} PtnShutdownFunction;

typedef struct {
    PtnValue callback;
    PtnValue *args;
    size_t argc;
} PtnTickFunction;

typedef struct {
    size_t function_index;
    const char *name;
    PtnReference *reference;
} PtnStaticLocalSlot;

typedef struct {
    size_t object_id;
    size_t effective_object_id;
    char *property;
    size_t property_len;
    int operation;
} PtnMagicPropertyFrame;

typedef enum {
    PTN_MAGIC_PROPERTY_ISSET = 1,
    PTN_MAGIC_PROPERTY_GET = 2,
    PTN_MAGIC_PROPERTY_SET = 3,
    PTN_MAGIC_PROPERTY_UNSET = 4
} PtnMagicPropertyOperation;

typedef enum {
    PTN_STREAM_BACKEND_FILE,
    PTN_STREAM_BACKEND_MEMORY,
    PTN_STREAM_BACKEND_INPUT,
    PTN_STREAM_BACKEND_TEMP,
    PTN_STREAM_BACKEND_OUTPUT,
    PTN_STREAM_BACKEND_PIPE,
    PTN_STREAM_BACKEND_RFC2397,
    PTN_STREAM_BACKEND_ZLIB
} PtnStreamBackend;

typedef struct {
    unsigned char *data;
    size_t len;
    size_t capacity;
    size_t position;
    size_t max_memory;
    int writable;
    int append;
    int spilled;
    int eof;
    int error;
} PtnMemoryStream;

typedef enum {
    PTN_NUMBER_INT,
    PTN_NUMBER_FLOAT
} PtnNumberType;

typedef struct {
    PtnNumberType type;
    int64_t integer;
    double floating;
} PtnNumber;

typedef struct {
    const char *data;
    char *owned;
    size_t len;
} PtnStringOperand;

typedef struct {
    PtnValue value;
    size_t line;
} PtnConcatOperand;

struct PtnException {
    size_t refcount;
    size_t object_id;
    PtnRuntime *lifecycle_runtime;
    const char *class_name;
    char *message;
    size_t message_len;
    char *uncaught_text;
    size_t uncaught_text_len;
    int64_t code;
    const char *path;
    size_t line;
    int message_defined_at_location;
    PtnValue trace;
    PtnValue previous;
    int64_t severity;
    PtnValue dynamic_properties;
    PtnValue errors;
    PtnValue soap_fault_headerfault;
};

typedef void (*PtnResourceCloseHook)(PtnResource *resource, void *data);
typedef void (*PtnResourceHookDataFree)(void *data);

struct PtnResource {
    size_t refcount;
    int64_t id;
    const char *type_name;
    FILE *stream;
    void *directory;
    char *stream_uri;
    char *stream_mode;
    PtnStreamBackend stream_backend;
    PtnMemoryStream *memory_stream;
    PtnStreamFilter *read_filters;
    PtnStreamFilter *write_filters;
    char *filtered_read_buffer;
    size_t filtered_read_buffer_len;
    size_t filtered_read_buffer_offset;
    size_t chunk_size;
    PtnResourceCloseHook close_hook;
    void *close_hook_data;
    PtnResourceHookDataFree close_hook_data_free;
    int persistent;
    int closed;
    PtnValue context_options;
    PtnValue context_params;
    PtnValue curl_options;
    PtnResource *registry_prev;
    PtnResource *registry_next;
    int manual_close_forbidden;
    size_t object_id;
    PtnRuntime *object_id_runtime;
};

struct PtnStreamFilter {
    PtnStreamFilterKind kind;
    char *name;
    int base64_values[4];
    size_t base64_value_count;
    size_t filter_line_length;
    size_t filter_line_position;
    char *filter_line_break;
    size_t filter_line_break_len;
    int filter_line_break_configured;
    int quoted_printable_invalid_sequence;
    char *iconv_from_encoding;
    char *iconv_to_encoding;
    char *iconv_from_display;
    char *iconv_to_display;
    int iconv_error;
    size_t dechunk_remaining;
    size_t dechunk_size;
    int dechunk_size_seen;
    int dechunk_state;
    int64_t zlib_window;
    int64_t zlib_level;
    int zlib_error;
    int write_seek_mode;
    int user_filter_invalid_callback_reported;
    int user_filter_closed;
    int has_user_filter_object;
    PtnValue user_filter_object;
    PtnRuntime *user_filter_runtime;
    size_t user_filter_line;
    PtnStreamFilter *next;
};

typedef void (*PtnStreamFilterChainFlushClosingHook)(PtnStreamFilter *filter);
static PtnStreamFilterChainFlushClosingHook ptn_stream_filter_chain_flush_closing_hook = NULL;

typedef struct {
    int has_handler;
    PtnValue handler;
    int64_t levels;
} PtnErrorHandlerFrame;

typedef struct {
    int has_handler;
    PtnValue handler;
} PtnExceptionHandlerFrame;

typedef struct {
    PtnException *active_exception;
    PtnTryFrame *try_frame;
    int has_exception_handler;
    PtnValue exception_handler;
    PtnExceptionHandlerFrame *exception_handler_stack;
    size_t exception_handler_stack_len;
    size_t exception_handler_stack_capacity;
    int in_exception_handler;
} PtnExceptionState;

typedef struct {
    size_t argc;
    const PtnValue *args;
    const char *const *arg_names;
    size_t parameter_count;
    const char *const *parameter_names;
    int has_current_closure;
    PtnValue current_closure;
} PtnCallFrame;

struct PtnTraceFrame {
    PtnRuntime *runtime;
    const char *function_name;
    const char *file;
    size_t line;
    size_t argc;
    const PtnValue *args;
    const char *const *arg_names;
    size_t parameter_count;
    const char *const *parameter_names;
    size_t sensitive_parameter_count;
    const unsigned char *sensitive_parameters;
    size_t sensitive_variadic_position;
    int has_receiver;
    PtnValue receiver;
    PtnTraceFrame *previous;
};

struct PtnTryFrame {
    jmp_buf jump;
    PtnTryFrame *previous;
    int is_user_try;
};

typedef struct {
    PtnRuntime *runtime;
    FILE *stream;
    int emitted_deprecation;
    int emitted_warning;
    int suppressed;
    int64_t error_reporting;
    int display_errors;
    int html_errors;
    char *html_errors_ini_value;
    int last_error_set;
    int64_t last_error_type;
    char *last_error_message;
    char *last_error_file;
    size_t last_error_line;
    int has_error_handler;
    PtnValue error_handler;
    int64_t error_handler_levels;
    int error_handler_call_depth;
    PtnErrorHandlerFrame *error_handler_stack;
    size_t error_handler_stack_len;
    size_t error_handler_stack_capacity;
} PtnDiagnosticSink;

typedef PtnValue (*PtnMethodDispatchHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *method_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
typedef int (*PtnReflectedMethodDispatchHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *target_class_name,
    const char *method_name,
    const char *called_class_name,
    size_t argc,
    const PtnValue *args,
    size_t line,
    PtnValue *result_out
);
typedef int (*PtnDeclaredMethodExistsHandler)(const char *class_name, const char *method_name);
typedef PtnFunctionMetadata (*PtnDeclaredMethodMetadataHandler)(
    const char *class_name,
    const char *method_name
);
typedef int (*PtnDeclaredMethodVisibilityHandler)(
    int visibility,
    const char *declaring_class,
    const char *target_class_name,
    const char *method_name,
    const char *access_scope
);
typedef int (*PtnDeclaredMethodVisibilityMetadataHandler)(
    const char *class_name,
    const char *method_name,
    const char **declaring_class,
    int *visibility,
    int *is_abstract
);
typedef int (*PtnClassScopeAllowsHandler)(
    const char *access_scope,
    const char *declaring_class
);
typedef int (*PtnDeclaredClassReadonlyHandler)(const char *class_name);
typedef int (*PtnDeclaredClassAllowsDynamicPropertiesHandler)(const char *class_name);
typedef int (*PtnMagicPropertyReadHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    size_t line,
    int require_isset,
    PtnValue *value_out
);
typedef int (*PtnMagicPropertyIssetHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line,
    int *isset_out
);
typedef int (*PtnMagicPropertyGetHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line,
    PtnValue *value_out
);
typedef int (*PtnMagicPropertyGetExistsHandler)(PtnRuntime *runtime, PtnValue receiver);
typedef int (*PtnMagicPropertySetHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    PtnValue value,
    size_t line
);
typedef int (*PtnMagicPropertyUnsetHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    size_t line
);
typedef int (*PtnMagicDebugInfoHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t line,
    PtnValue *value_out
);
typedef int (*PtnPropertyHookGetHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *class_name,
    const char *property_name,
    size_t line,
    PtnValue *value_out
);
typedef int (*PtnPropertyHookSetHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *class_name,
    const char *property_name,
    PtnValue value,
    size_t line
);
typedef int (*PtnClassConstantInitializerHandler)(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant_name
);
typedef int (*PtnStaticPropertyInitializerHandler)(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property_name
);
typedef PtnValue (*PtnNewInstanceWithoutConstructorHandler)(
    PtnRuntime *runtime,
    const char *class_name,
    size_t line
);

struct PtnRuntime {
    PtnSymbolTable symbols;
    PtnSymbolTable *global_symbols;
    PtnSymbolTable owned_constants;
    PtnSymbolTable *constants;
    PtnSymbolTable owned_constant_sources;
    PtnSymbolTable *constant_sources;
    PtnSymbolTable owned_class_aliases;
    PtnSymbolTable *class_aliases;
    PtnSymbolTable owned_dynamic_classes;
    PtnSymbolTable *dynamic_classes;
    PtnSymbolTable owned_class_constants;
    PtnSymbolTable *class_constants;
    PtnSymbolTable owned_class_constant_deprecations;
    PtnSymbolTable *class_constant_deprecations;
    PtnSymbolTable owned_class_constant_initializing;
    PtnSymbolTable *class_constant_initializing;
    const char *current_class_constant_initializing_class_name;
    const char *current_class_constant_initializing_key_class_name;
    const char *current_class_constant_initializing_constant_name;
    const char *current_class_constant_source_path;
    const char *class_constant_deprecation_suppress_class;
    const char *class_constant_deprecation_suppress_constant;
    PtnObject *dynamic_property_deprecation_suppress_object;
    char *dynamic_property_deprecation_suppress_property;
    PtnSymbolTable owned_static_properties;
    PtnSymbolTable *static_properties;
    PtnSymbolTable owned_static_property_initialized;
    PtnSymbolTable *static_property_initialized;
    PtnSymbolTable owned_static_property_read_visibility;
    PtnSymbolTable *static_property_read_visibility;
    PtnSymbolTable owned_static_property_set_visibility;
    PtnSymbolTable *static_property_set_visibility;
    PtnSymbolTable owned_static_property_type_kind;
    PtnSymbolTable *static_property_type_kind;
    PtnSymbolTable owned_static_property_type_class_name;
    PtnSymbolTable *static_property_type_class_name;
    PtnSymbolTable owned_static_property_type_text;
    PtnSymbolTable *static_property_type_text;
    PtnSymbolTable owned_static_property_type_allows_null;
    PtnSymbolTable *static_property_type_allows_null;
    PtnDiagnosticSink diagnostics;
    PtnExceptionState owned_exceptions;
    PtnExceptionState *exceptions;
    PtnCallFrame owned_call_frame;
    PtnCallFrame *call_frame;
    const char *const *next_call_arg_names;
    PtnTraceFrame owned_trace_frame;
    PtnTraceFrame *trace_frame;
    PtnRuntime *lifecycle_root;
    PtnObject **live_objects;
    size_t live_objects_len;
    size_t live_objects_capacity;
    PtnClosure **live_closures;
    size_t live_closures_len;
    size_t live_closures_capacity;
    PtnValue *first_class_callable_cache_values;
    char **first_class_callable_cache_names;
    size_t first_class_callable_cache_len;
    size_t first_class_callable_cache_capacity;
    PtnArray **live_arrays;
    size_t live_arrays_len;
    size_t live_arrays_capacity;
    PtnReference **live_references;
    size_t live_references_len;
    size_t live_references_capacity;
    size_t live_weak_maps_len;
    PtnValue *temporary_roots;
    size_t temporary_roots_len;
    size_t temporary_roots_capacity;
    PtnStaticLocalSlot *static_local_slots;
    size_t static_local_slots_len;
    size_t static_local_slots_capacity;
    size_t next_object_id;
    size_t *free_object_ids;
    size_t free_object_ids_len;
    size_t free_object_ids_capacity;
    size_t deferred_free_object_id;
    int has_deferred_free_object_id;
    PtnOutputBuffer *output_buffers;
    size_t output_buffers_len;
    size_t output_buffers_capacity;
    size_t output_buffer_callback_depth;
    const char *output_buffer_callback_function_name;
    char *output_buffer_callback_handler_name;
    size_t output_buffer_callback_line;
    int output_buffer_callback_output_warned;
    int output_buffer_callback_passthrough_output;
    size_t output_buffer_callback_skip_buffers;
    int output_at_line_start;
    int output_has_started;
    int http_response_code_initialized;
    int64_t http_response_code;
    int header_callback_registered;
    int header_callback_running;
    int header_callback_completed;
    PtnValue header_callback;
    PtnShutdownFunction *shutdown_functions;
    size_t shutdown_functions_len;
    size_t shutdown_functions_capacity;
    size_t shutdown_function_index;
    int shutdown_functions_running;
    int shutdown_functions_completed;
    int shutdown_in_progress;
    int tick_enabled;
    PtnTickFunction *tick_functions;
    size_t tick_functions_len;
    size_t tick_functions_capacity;
    int tick_functions_running;
    int defer_uncaught_exception_emit;
    PtnMethodDispatchHandler method_dispatch;
    PtnReflectedMethodDispatchHandler reflected_method_dispatch;
    PtnDeclaredMethodExistsHandler declared_method_exists;
    PtnDeclaredMethodMetadataHandler declared_method_metadata;
    PtnDeclaredMethodVisibilityHandler declared_method_visible;
    PtnDeclaredMethodVisibilityMetadataHandler declared_method_visibility_metadata;
    PtnClassScopeAllowsHandler class_scope_allows;
    PtnDeclaredClassReadonlyHandler declared_class_is_readonly;
    PtnDeclaredClassAllowsDynamicPropertiesHandler declared_class_allows_dynamic_properties;
    PtnMagicPropertyReadHandler magic_property_read;
    PtnMagicPropertyIssetHandler magic_property_isset;
    int *declared_user_functions;
    int *declared_user_classes;
    int *declared_user_traits;
    PtnMagicPropertyGetHandler magic_property_get;
    PtnMagicPropertyGetExistsHandler magic_property_get_exists;
    PtnMagicPropertySetHandler magic_property_set;
    PtnMagicPropertyUnsetHandler magic_property_unset;
    PtnMagicDebugInfoHandler magic_debug_info;
    PtnPropertyHookGetHandler property_hook_get;
    PtnPropertyHookSetHandler property_hook_set;
    const char *active_property_hook_class;
    const char *active_property_hook_property;
    PtnObject *active_property_hook_object;
    PtnClassConstantInitializerHandler class_constant_initializer;
    PtnStaticPropertyInitializerHandler static_property_initializer;
    PtnNewInstanceWithoutConstructorHandler new_instance_without_constructor;
    int in_magic_property_dispatch;
    size_t active_spl_object_storage_get_hash_depth;
    PtnMagicPropertyFrame *magic_property_frames;
    size_t magic_property_frame_len;
    size_t magic_property_frame_capacity;
    const char *source_path;
    const unsigned char *source_snapshot_data;
    size_t source_snapshot_len;
    size_t compiled_include_depth;
    int in_preload;
    const char *current_function_name;
    const char *current_class_name;
    const char *current_called_class_name;
    const char *called_class_name_override;
    const char *forward_static_called_class_name;
    const char *destructor_access_scope;
    int destructor_shutdown_phase;
    PtnGenerator *current_generator;
    const char *pending_generator_assignment_name;
    PtnGenerator *pending_yield_from_generator;
    size_t pending_yield_from_line;
    int implicit_generator_foreach_rewind;
    const char *implicit_generator_foreach_source_path;
    size_t implicit_generator_foreach_line;
    int generator_aborted_after_yield;
    int generator_aborted_rethrow_on_rewind;
    int generator_chained_exception_during_unwind;
    int defer_unreferenced_destructors_for_catch;
    PtnValue deferred_yield_from_iterator_object;
    int suppress_generator_rewind_trace_frame;
    PtnObject *current_fiber;
    int has_current_receiver;
    PtnValue current_receiver;
    const char *by_ref_argument_function_name_override;
    int by_ref_argument_notice_pending;
    int by_ref_argument_notice_emitted;
    size_t by_ref_argument_notice_line;
    int suppress_scoped_callable_deprecation;
    char *include_path;
    char **included_files;
    size_t included_files_len;
    size_t included_files_capacity;
    PtnValue *autoload_callbacks;
    char **autoload_callback_scope_class_names;
    char **autoload_callback_called_class_names;
    size_t autoload_callbacks_len;
    size_t autoload_callbacks_capacity;
    char *spl_autoload_extensions;
    char **autoloading_class_names;
    size_t autoloading_class_names_len;
    size_t autoloading_class_names_capacity;
    PtnResource *last_opened_directory;
    char *open_basedir;
    char *memory_limit;
    char *max_memory_limit;
    char *auto_detect_line_endings;
    char *default_charset;
    char *arg_separator_input;
    char *arg_separator_output;
    char *highlight_comment;
    char *highlight_default;
    char *highlight_html;
    char *highlight_keyword;
    char *highlight_string;
    char *output_handler;
    char *filter_default;
    char *pcre_backtrack_limit;
    char *pcre_recursion_limit;
    char *pcre_jit;
    char *opcache_blacklist_filename;
    char *opcache_enable;
    char *opcache_enable_cli;
    char *opcache_fast_shutdown;
    char *opcache_file_cache_only;
    char *opcache_file_update_protection;
    char *opcache_interned_strings_buffer;
    char *opcache_log_verbosity_level;
    char *opcache_optimization_level;
    char *opcache_opt_debug_level;
    char *opcache_preload;
    char *opcache_preload_user;
    char *opcache_save_comments;
    char *opcache_validate_timestamps;
    char *phar_readonly;
    char *phar_require_hash;
    char *phar_cache_list;
    char *internal_encoding;
    char *input_encoding;
    char *output_encoding;
    char *iconv_internal_encoding;
    char *iconv_input_encoding;
    char *iconv_output_encoding;
    int date_timezone_startup_warning_emitted;
    char *variables_order;
    char *register_argc_argv;
    char *enable_post_data_reading;
    int native_argc;
    char **native_argv;
    char *file_uploads;
    char *max_input_vars;
    char *max_input_nesting_level;
    char *post_max_size;
    char *always_populate_raw_post_data;
    char *upload_tmp_dir;
    char *expose_php;
    char *docref_root;
    char *user_agent;
    char *unserialize_callback_func;
    int unserialize_max_depth;
    char *request_body;
    size_t request_body_len;
    PtnSymbolTable session_ini;
    char *session_id;
    int session_active;
    int session_was_started;
    int session_auto_started;
    const char *session_start_path;
    size_t session_start_line;
    int session_save_handler_kind;
    PtnValue session_save_handler_object;
    PtnValue session_save_handler_callbacks[9];
    int session_save_handler_register_shutdown;
    int session_save_handler_in_callback;
    int session_save_handler_shutdown_warning_pending;
    int session_parent_handler_open;
    char *session_parent_save_handler;
    int session_lazy_write;
    char *session_last_data;
    size_t session_last_data_len;
    int session_last_data_valid;
    int precision;
    int serialize_precision;
    int initial_precision;
    int initial_serialize_precision;
    int bcmath_scale;
    int initial_bcmath_scale;
    int exception_ignore_args;
    size_t exception_string_param_max_len;
    int strict_types;
    int initial_zend_assertions;
    int zend_assertions;
    int assert_active;
    int assert_warning;
    int assert_bail;
    char *assert_callback_ini;
    PtnValue assert_callback;
    int assert_exception;
    char *disabled_functions;
    size_t call_site_line;
    int suppress_user_call_frame_location;
    int suppress_user_argument_count_location;
    int warn_by_ref_argument_mismatch;
    int throw_argument_count_errors;
    int gc_enabled;
    int gc_running;
    size_t gc_mark_epoch;
    size_t gc_runs;
    size_t gc_collected;
    size_t gc_roots;
    void *active_serialize_state;
    void *active_unserialize_state;
    char *strtok_string;
    size_t strtok_len;
    size_t strtok_offset;
    int strtok_has_state;
    int json_last_error;
    size_t json_last_error_line;
    size_t json_last_error_column;
    int pcre_last_error;
    const char *pcre_utf8_cache_data;
    size_t pcre_utf8_cache_len;
    int pcre_utf8_cache_known;
    int pcre_utf8_cache_valid;
    int intl_error_level;
    int intl_use_exceptions;
    char *intl_last_error_message;
};

typedef struct {
    size_t string_allocs;
    size_t string_frees;
    size_t string_clones;
    size_t string_retain;
    size_t string_release;
    size_t string_detaches;
    size_t array_allocs;
    size_t array_retain;
    size_t array_release;
    size_t array_frees;
    size_t array_clones;
    size_t array_detaches;
    size_t array_detach_skips;
} PtnCowDebugCounters;

static PtnCowDebugCounters ptn_cow_debug_counters;

static PTN_UNUSED int ptn_is_truthy(PtnValue value);
static PTN_UNUSED void ptn_value_destroy(PtnValue *value);
static PTN_UNUSED void ptn_value_destroy_with_runtime_scope(PtnRuntime *runtime, PtnValue *value);
static PTN_UNUSED void ptn_value_destroy_with_runtime_scope_at(PtnRuntime *runtime, PtnValue *value, size_t line);
static PTN_UNUSED void ptn_symbols_free_with_runtime_scope(PtnSymbolTable *symbols, PtnRuntime *runtime);
static void ptn_runtime_free(PtnRuntime *runtime);
static PTN_UNUSED void ptn_exception_free(PtnException *exception);
static PTN_UNUSED void ptn_reference_release(PtnReference *reference);
static void ptn_abort_out_of_memory(void);
static PTN_UNUSED int ptn_ascii_case_equal(const char *left, const char *right);
static PTN_UNUSED int ptn_object_is_generator(PtnObject *object);
static PTN_UNUSED int ptn_object_is_incomplete_class(PtnObject *object);
static PTN_UNUSED void ptn_throw_incomplete_object_method_call(
    PtnRuntime *runtime,
    PtnObject *object,
    size_t line
);
static PTN_UNUSED PtnValue ptn_generator_current(PtnRuntime *runtime, PtnValue receiver, size_t line);
static PTN_UNUSED void ptn_generator_force_close(PtnRuntime *runtime, PtnGenerator *generator);
static PTN_UNUSED PtnValue ptn_generator_get_return(PtnRuntime *runtime, PtnValue receiver, size_t line);
static PTN_UNUSED PtnValue ptn_generator_key(PtnRuntime *runtime, PtnValue receiver, size_t line);
static PTN_UNUSED PtnValue ptn_generator_next(PtnRuntime *runtime, PtnValue receiver, size_t line);
static PTN_UNUSED int ptn_generator_capture_pending_exception(PtnRuntime *runtime, PtnGenerator *generator);
static PTN_UNUSED PtnValue ptn_generator_rewind(PtnRuntime *runtime, PtnValue receiver, size_t line);
static PTN_UNUSED void ptn_generator_trace_set_file_line(PtnValue frame, const char *file, size_t line);
static PTN_UNUSED void ptn_generator_register_send_call(PtnRuntime *runtime, const char *function_name, size_t argc, const PtnValue *args, size_t yield_argc, const size_t *yield_indexes, size_t line);
static PTN_UNUSED void ptn_generator_register_send_callable(PtnRuntime *runtime, PtnValue callable, size_t argc, const PtnValue *args, size_t yield_argc, const size_t *yield_indexes, size_t line);
static PTN_UNUSED void ptn_generator_register_send_method(PtnRuntime *runtime, PtnValue receiver, const char *method_name, size_t argc, const PtnValue *args, size_t yield_argc, const size_t *yield_indexes, size_t line);
static PTN_UNUSED void ptn_generator_register_send_nested_call(PtnRuntime *runtime, const char *outer_function_name, const char *inner_function_name, size_t argc, const PtnValue *args, size_t yield_argc, const size_t *yield_indexes, size_t line);
static PTN_UNUSED void ptn_generator_register_send_yield_from(PtnRuntime *runtime, size_t line);
static PTN_UNUSED PtnValue ptn_generator_send(PtnRuntime *runtime, PtnValue receiver, PtnValue sent_value, size_t line);
static PTN_UNUSED void ptn_generator_set_return_value(PtnRuntime *runtime, PtnGenerator *generator, PtnValue value);
static PTN_UNUSED PtnValue ptn_generator_throw(PtnRuntime *runtime, PtnValue receiver, PtnValue exception, size_t line);
static PTN_UNUSED PtnValue ptn_generator_valid(PtnRuntime *runtime, PtnValue receiver, size_t line);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED PtnValue ptn_call_function(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line);
#endif
static PTN_UNUSED char *ptn_duplicate_string(const char *string);
static PTN_UNUSED PtnStringOperand ptn_runtime_global_constant_key_len(const char *name, size_t name_len);
static PTN_UNUSED char *ptn_runtime_global_constant_key(const char *name);
static PTN_UNUSED void ptn_string_operand_free(PtnStringOperand operand);
static PTN_UNUSED char *ptn_value_to_string(PtnValue value);
static PTN_UNUSED void ptn_output_write(PtnRuntime *runtime, const char *data, size_t len);
static PTN_UNUSED int ptn_declared_class_exists(const char *name);
static PTN_UNUSED PtnValue ptn_declared_class_new_instance_without_constructor(PtnRuntime *caller_runtime, const char *class_name, size_t line);
static PTN_UNUSED int ptn_declared_runtime_class_exists(PtnRuntime *runtime, const char *name);
static PTN_UNUSED int ptn_declared_runtime_user_class_exists(PtnRuntime *runtime, const char *name);
static PTN_UNUSED int ptn_declared_runtime_interface_exists(PtnRuntime *runtime, const char *name);
static PTN_UNUSED int ptn_declared_runtime_trait_exists(PtnRuntime *runtime, const char *name);
static PTN_UNUSED int ptn_declared_runtime_class_is_linking(PtnRuntime *runtime, const char *name);
static PTN_UNUSED void ptn_declared_runtime_class_mark_variance_dependency(PtnRuntime *runtime, const char *name);
static PTN_UNUSED int ptn_declared_runtime_class_slot_has_variance_dependency(PtnRuntime *runtime, size_t index);
static PTN_UNUSED int ptn_declared_runtime_variance_type_available(PtnRuntime *runtime, const char *name, size_t line);
static PTN_UNUSED int ptn_declared_user_class_or_interface_exists(const char *name);
static PTN_UNUSED const char *ptn_declared_class_canonical_name(const char *name);
static PTN_UNUSED int ptn_declared_class_is_final(const char *name);
static PTN_UNUSED const char *ptn_builtin_exception_class_name(const char *class_name);
static PTN_UNUSED void ptn_emit_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line);
static PTN_UNUSED void ptn_emit_user_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line);
static PTN_UNUSED void ptn_diagnostic_output_write(PtnDiagnosticSink *diagnostics, const char *data, size_t len);
static PTN_UNUSED void ptn_diagnostic_output_html_text_len(PtnDiagnosticSink *diagnostics, const char *data, size_t len);
static PTN_UNUSED void ptn_throw_exception(PtnRuntime *runtime, const char *class_name, const char *message);
static PTN_UNUSED void ptn_rethrow_exception(PtnRuntime *runtime);
static PTN_UNUSED void ptn_try_frame_push(PtnRuntime *runtime, PtnTryFrame *frame);
static PTN_UNUSED void ptn_try_frame_pop(PtnRuntime *runtime, PtnTryFrame *frame);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED PtnValue ptn_call_callable(PtnRuntime *runtime, PtnValue callable, size_t argc, const PtnValue *args, size_t line, int from_call_user_func);
static PTN_UNUSED PtnValue ptn_call_declared_method(PtnRuntime *runtime, PtnValue receiver, const char *method_name, size_t argc, const PtnValue *args, size_t line);
#endif
static PTN_UNUSED int ptn_declared_class_direct_non_private_method_exists(const char *class_name, const char *method_name);
static void ptn_symbols_free(PtnSymbolTable *symbols);
static PTN_UNUSED void ptn_cow_debug_note_string_alloc(void);
static PTN_UNUSED void ptn_cow_debug_note_string_free(void);
static PTN_UNUSED void ptn_cow_debug_note_string_clone(void);
static PTN_UNUSED void ptn_cow_debug_note_string_retain(void);
static PTN_UNUSED void ptn_cow_debug_note_string_release(void);
static PTN_UNUSED void ptn_cow_debug_note_string_detach(void);
static PTN_UNUSED void ptn_cow_debug_note_array_alloc(void);
static PTN_UNUSED void ptn_cow_debug_note_array_retain(void);
static PTN_UNUSED void ptn_cow_debug_note_array_release(void);
static PTN_UNUSED void ptn_cow_debug_note_array_free(void);
static PTN_UNUSED void ptn_cow_debug_note_array_clone(void);
static PTN_UNUSED void ptn_cow_debug_note_array_detach(void);
static PTN_UNUSED void ptn_cow_debug_note_array_detach_skip(void);
static PTN_UNUSED void ptn_cow_debug_assert_string_refcount(size_t *refcount, const char *operation);
static PTN_UNUSED void ptn_cow_debug_assert_array_refcount(PtnArray *array, const char *operation);
static PTN_UNUSED void ptn_cow_debug_reset(void);
static PTN_UNUSED int ptn_cow_debug_counter(const char *name, size_t *out);
static PTN_UNUSED void ptn_cow_debug_assert_named_counter(const char *name, int64_t expected);
static PTN_UNUSED void ptn_cow_debug_assert_balanced(void);
static PTN_UNUSED void ptn_output_buffer_flush_all(PtnRuntime *runtime);
static PTN_UNUSED int ptn_runtime_memory_limit_bytes(PtnRuntime *runtime, size_t *limit_out);
static PTN_UNUSED void ptn_runtime_run_object_destructors_until_output_buffer(PtnRuntime *runtime);
static PTN_UNUSED void ptn_runtime_run_unreferenced_object_destructors(PtnRuntime *runtime);
static PTN_UNUSED void ptn_runtime_run_object_destructors(PtnRuntime *runtime);
static PTN_UNUSED const char *ptn_runtime_resolve_class_alias(
    PtnRuntime *runtime,
    const char *class_name
);
static PTN_UNUSED void ptn_runtime_autoload_class(
    PtnRuntime *runtime,
    const char *class_name,
    size_t line
);
static PtnSymbolTable *ptn_runtime_class_alias_table(PtnRuntime *runtime);
static PTN_UNUSED int ptn_runtime_dynamic_class_exists(PtnRuntime *runtime, const char *class_name);
static PTN_UNUSED void ptn_runtime_register_dynamic_class(PtnRuntime *runtime, const char *class_name);
static PTN_UNUSED void ptn_runtime_register_dynamic_class_with_parent(
    PtnRuntime *runtime,
    const char *class_name,
    const char *parent_name
);
static PTN_UNUSED const char *ptn_runtime_declared_class_parent_name(
    PtnRuntime *runtime,
    const char *class_name
);
static PTN_UNUSED int ptn_runtime_declared_class_is_same_or_descendant(
    PtnRuntime *runtime,
    const char *class_name,
    const char *ancestor_name
);

typedef PtnValue (*PtnInternalFunctionHandler)(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);

typedef struct {
    const char *name;
    size_t min_args;
    size_t max_args;
    PtnInternalFunctionHandler handler;
} PtnInternalFunction;

#define PTN_VARIADIC_ARGS ((size_t)-1)

static PTN_UNUSED PtnFunctionMetadata ptn_function_metadata_not_found(void) {
    PtnFunctionMetadata metadata;
    metadata.found = 0;
    metadata.name = NULL;
    metadata.is_internal = 0;
    metadata.parameter_count = 0;
    metadata.required_parameter_count = 0;
    metadata.is_variadic = 0;
    metadata.parameters = NULL;
    metadata.return_by_ref = 0;
    metadata.is_generator = 0;
    metadata.is_deprecated = 0;
    metadata.return_type_name = NULL;
    metadata.return_type_display_name = NULL;
    metadata.return_type_allows_null = 0;
    metadata.return_type_is_builtin = 0;
    metadata.tentative_return_type_name = NULL;
    metadata.tentative_return_type_display_name = NULL;
    metadata.tentative_return_type_allows_null = 0;
    metadata.tentative_return_type_is_builtin = 0;
    metadata.source_file = NULL;
    metadata.start_line = 0;
    metadata.end_line = 0;
    metadata.doc_comment = NULL;
    metadata.static_variables_provider = NULL;
    metadata.has_user_function_index = 0;
    metadata.user_function_index = 0;
    metadata.attribute_method_name = NULL;
    return metadata;
}

static PTN_UNUSED PtnFunctionMetadata ptn_function_metadata_found(
    const char *name,
    int is_internal,
    size_t parameter_count,
    size_t required_parameter_count,
    int is_variadic,
    const PtnParameterMetadata *parameters,
    int return_by_ref,
    const char *return_type_name,
    const char *return_type_display_name,
    int return_type_allows_null,
    int return_type_is_builtin
) {
    PtnFunctionMetadata metadata;
    metadata.found = 1;
    metadata.name = name;
    metadata.is_internal = is_internal;
    metadata.parameter_count = parameter_count;
    metadata.required_parameter_count = required_parameter_count;
    metadata.is_variadic = is_variadic;
    metadata.parameters = parameters;
    metadata.return_by_ref = return_by_ref;
    metadata.is_generator = 0;
    metadata.is_deprecated = 0;
    metadata.return_type_name = return_type_name;
    metadata.return_type_display_name = return_type_display_name;
    metadata.return_type_allows_null = return_type_allows_null;
    metadata.return_type_is_builtin = return_type_is_builtin;
    metadata.tentative_return_type_name = NULL;
    metadata.tentative_return_type_display_name = NULL;
    metadata.tentative_return_type_allows_null = 0;
    metadata.tentative_return_type_is_builtin = 0;
    metadata.source_file = NULL;
    metadata.start_line = 0;
    metadata.end_line = 0;
    metadata.doc_comment = NULL;
    metadata.static_variables_provider = NULL;
    metadata.has_user_function_index = 0;
    metadata.user_function_index = 0;
    metadata.attribute_method_name = NULL;
    return metadata;
}

static PTN_UNUSED PtnFunctionMetadata ptn_function_metadata_with_tentative_return(
    PtnFunctionMetadata metadata
) {
    metadata.tentative_return_type_name = metadata.return_type_name;
    metadata.tentative_return_type_display_name = metadata.return_type_display_name;
    metadata.tentative_return_type_allows_null = metadata.return_type_allows_null;
    metadata.tentative_return_type_is_builtin = metadata.return_type_is_builtin;
    metadata.return_type_name = NULL;
    metadata.return_type_display_name = NULL;
    metadata.return_type_allows_null = 0;
    metadata.return_type_is_builtin = 0;
    return metadata;
}

static PTN_UNUSED PtnFunctionMetadata ptn_function_metadata_with_flags(
    PtnFunctionMetadata metadata,
    int is_generator,
    int is_deprecated
) {
    metadata.is_generator = is_generator;
    metadata.is_deprecated = is_deprecated;
    return metadata;
}

static PTN_UNUSED PtnFunctionMetadata ptn_function_metadata_with_source(
    PtnFunctionMetadata metadata,
    const char *source_file,
    size_t start_line,
    size_t end_line,
    const char *doc_comment,
    PtnFunctionStaticVariablesProvider static_variables_provider
) {
    metadata.source_file = source_file;
    metadata.start_line = start_line;
    metadata.end_line = end_line;
    metadata.doc_comment = doc_comment;
    metadata.static_variables_provider = static_variables_provider;
    return metadata;
}

static PTN_UNUSED PtnFunctionMetadata ptn_function_metadata_with_user_function_index(
    PtnFunctionMetadata metadata,
    size_t user_function_index
) {
    metadata.has_user_function_index = 1;
    metadata.user_function_index = user_function_index;
    return metadata;
}

static PTN_UNUSED PtnFunctionMetadata ptn_function_metadata_with_attribute_method(
    PtnFunctionMetadata metadata,
    const char *attribute_method_name
) {
    metadata.attribute_method_name = attribute_method_name;
    return metadata;
}

static PTN_UNUSED PtnRuntime *ptn_runtime_root(PtnRuntime *runtime) {
    if (runtime == NULL) {
        return NULL;
    }
    return runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
}

static PTN_UNUSED void ptn_runtime_shutdown_before_exit(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root != NULL) {
        ptn_runtime_free(root);
    }
}

static PTN_UNUSED int ptn_runtime_has_included_file(PtnRuntime *runtime, const char *path) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL || path == NULL) {
        return 0;
    }
    for (size_t i = 0; i < root->included_files_len; i++) {
        if (strcmp(root->included_files[i], path) == 0) {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED void ptn_runtime_note_included_file(PtnRuntime *runtime, const char *path) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL || path == NULL) {
        return;
    }
    for (size_t i = 0; i < root->included_files_len; i++) {
        if (strcmp(root->included_files[i], path) == 0) {
            return;
        }
    }
    if (root->included_files_len == root->included_files_capacity) {
        size_t new_capacity = root->included_files_capacity == 0
            ? 8
            : root->included_files_capacity * 2;
        if (new_capacity < root->included_files_capacity ||
            new_capacity > SIZE_MAX / sizeof(char *)) {
            ptn_abort_out_of_memory();
        }
        char **new_files = realloc(root->included_files, new_capacity * sizeof(char *));
        if (new_files == NULL) {
            ptn_abort_out_of_memory();
        }
        root->included_files = new_files;
        root->included_files_capacity = new_capacity;
    }
    root->included_files[root->included_files_len++] = ptn_duplicate_string(path);
}

static void ptn_runtime_push_free_object_id(PtnRuntime *root, size_t object_id) {
    if (root->free_object_ids_len == root->free_object_ids_capacity) {
        size_t new_capacity = root->free_object_ids_capacity == 0
            ? 8
            : root->free_object_ids_capacity * 2;
        if (new_capacity < root->free_object_ids_capacity ||
            new_capacity > SIZE_MAX / sizeof(size_t)) {
            ptn_abort_out_of_memory();
        }
        size_t *new_ids = realloc(root->free_object_ids, new_capacity * sizeof(size_t));
        if (new_ids == NULL) {
            ptn_abort_out_of_memory();
        }
        root->free_object_ids = new_ids;
        root->free_object_ids_capacity = new_capacity;
    }
    root->free_object_ids[root->free_object_ids_len++] = object_id;
}

static PTN_UNUSED size_t ptn_runtime_alloc_object_id(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        return 0;
    }
    size_t object_id = 0;
    if (root->free_object_ids_len > 0) {
        object_id = root->free_object_ids[--root->free_object_ids_len];
    } else {
        if (root->next_object_id == 0) {
            root->next_object_id = 1;
        }
        if (root->next_object_id > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        object_id = root->next_object_id++;
    }
    if (root->has_deferred_free_object_id) {
        ptn_runtime_push_free_object_id(root, root->deferred_free_object_id);
        root->deferred_free_object_id = 0;
        root->has_deferred_free_object_id = 0;
    }
    return object_id;
}

static PTN_UNUSED void ptn_runtime_release_object_id(PtnRuntime *runtime, size_t object_id) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL || object_id == 0) {
        return;
    }
    ptn_runtime_push_free_object_id(root, object_id);
}

static PTN_UNUSED void ptn_runtime_release_object_id_after_next_allocation(
    PtnRuntime *runtime,
    size_t object_id
) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL || object_id == 0) {
        return;
    }
    if (root->has_deferred_free_object_id) {
        ptn_runtime_push_free_object_id(root, root->deferred_free_object_id);
    }
    root->deferred_free_object_id = object_id;
    root->has_deferred_free_object_id = 1;
}

static PTN_UNUSED void ptn_runtime_register_closure(PtnRuntime *runtime, PtnClosure *closure);
static PTN_UNUSED void ptn_runtime_unregister_closure(PtnRuntime *runtime, PtnClosure *closure);

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED int ptn_internal_class_name_is_reflection_class(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_attribute(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_object(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_enum(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_enum_unit_case(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_enum_backed_case(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_enum_case(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_extension(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_zend_extension(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_function(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_fiber(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_generator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_method(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_class_constant(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_named_type(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_type(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_parameter(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_property(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_constant(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_reference(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_array_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_empty_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_array_object(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_spl_fixed_array(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_append_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_callback_filter_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_recursive_callback_filter_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_filter_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_infinite_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_iterator_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_multiple_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_no_rewind_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_recursive_iterator_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_limit_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_recursive_array_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_spl_object_storage(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_spl_heap(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_spl_max_heap(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_spl_min_heap(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_spl_priority_queue(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_spl_doubly_linked_list(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_spl_queue(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_spl_stack(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_spl_file_info(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_spl_file_object(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_directory_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_regex_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_directory(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_sensitive_parameter(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_sensitive_parameter_value(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_fiber(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_weak_map(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_weak_reference(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_attribute(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_allow_dynamic_properties(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_delayed_target_validation(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_deprecated(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_no_discard(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_return_type_will_change(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_datetime_immutable(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_datetime_zone(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_date_interval(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_date_period(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_internal_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_bcmath_number(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_pdo(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_pdo_statement(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_pdo_exception(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_pdo_row(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_sqlite3(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_sqlite3_stmt(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_sqlite3_result(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_curl_file(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_phar(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_phar_file_info(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_random_randomizer(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_random_engine(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_zip_archive(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_soap_client(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_soap_server(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_soap_header(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_soap_var(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_soap_param(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_hash_context(const char *class_name);
static PTN_UNUSED PtnValue ptn_hash_context_clone(PtnRuntime *runtime, PtnValue source, size_t line);
static PTN_UNUSED int ptn_internal_class_name_is_session_handler(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_php_token(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_intl_calendar(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_intl_date_formatter(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_intl_timezone(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_intl_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_message_formatter(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_intl_list_formatter(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_locale(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_number_formatter(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_intl_number_range_formatter(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_collator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_resource_bundle(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_spoofchecker(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_uconverter(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_dom(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_xml_reader(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_xml_writer(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_xml_parser(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_simplexml(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_uri_rfc3986_uri(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_uri_whatwg_url(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_uri_comparison_mode(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_uri_whatwg_url_host_type(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_uri_whatwg_url_validation_error(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_uri_whatwg_url_validation_error_type(const char *class_name);
static int ptn_internal_class_exists_name(const char *class_name);
static int ptn_internal_interface_exists_name(const char *name);
static PTN_UNUSED int ptn_internal_class_method_exists(const char *class_name, const char *method_name);
static PTN_UNUSED int ptn_internal_class_static_method_exists(const char *class_name, const char *method_name);
static PTN_UNUSED PtnValue ptn_hash_context_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED int ptn_runtime_class_exists(PtnRuntime *runtime, const char *class_name);
static PTN_UNUSED int ptn_runtime_interface_exists(
    PtnRuntime *runtime,
    const char *interface_name
);
static PTN_UNUSED int ptn_runtime_class_or_interface_exists(
    PtnRuntime *runtime,
    const char *class_name
);
static PTN_UNUSED PtnValue ptn_reflection_class_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_curl_file_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_object_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_enum_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_enum_case_new(
    PtnRuntime *runtime,
    const char *reflection_class_name,
    int require_backed,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_extension_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_zend_extension_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_function_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_generator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_fiber_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_method_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_parameter_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_class_constant_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_constant_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_sensitive_parameter_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_sensitive_parameter_value_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_fiber_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_weak_reference_create(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_weak_reference_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_weak_map_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_weak_map_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED int ptn_weak_map_bind_reference(
    PtnRuntime *runtime,
    PtnValue receiver,
    PtnValue key_value,
    PtnValue reference,
    size_t line
);
static PTN_UNUSED int ptn_weak_map_offset_isset(
    PtnRuntime *runtime,
    PtnValue receiver,
    PtnValue key_value,
    size_t line
);
static PTN_UNUSED size_t ptn_runtime_collect_weak_map_cycles(PtnRuntime *runtime);
static PTN_UNUSED PtnValue ptn_reflection_property_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_attribute_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_allow_dynamic_properties_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_delayed_target_validation_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_deprecated_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_no_discard_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_return_type_will_change_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_random_randomizer_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_random_engine_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_random_randomizer_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_pdo_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_pdo_statement_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_sqlite3_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_sensitive_parameter_value_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PtnValue ptn_weak_reference_referent_value(PtnObject *object);
static PTN_UNUSED PtnValue ptn_reflection_class_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_class_constant_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_constant_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_attribute_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_method_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_type_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_parameter_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_property_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_reference_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_function_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_generator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_fiber_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_extension_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_zend_extension_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_sensitive_parameter_value_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_weak_reference_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_weak_map_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_attribute_metadata_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_pdo_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_pdo_statement_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_sqlite3_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_sqlite3_stmt_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_sqlite3_result_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_session_handler_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_session_handler_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_class_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_class_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_enum_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_enum_case_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_iterator_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_recursive_array_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_empty_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_empty_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_object_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_object_new_uninitialized(PtnRuntime *runtime);
static PTN_UNUSED int ptn_array_object_initialize(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_object_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_object_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_fixed_array_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_fixed_array_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_fixed_array_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED int ptn_spl_fixed_array_iterator_from_object(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    size_t line,
    PtnArrayIterator *out
);
static PTN_UNUSED int ptn_internal_cast_array_object(PtnValue value, PtnValue *array_out);
static PTN_UNUSED PtnValue ptn_spl_object_storage_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_object_storage_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_object_storage_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_heap_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_heap_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_heap_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_doubly_linked_list_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_doubly_linked_list_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_doubly_linked_list_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED int ptn_spl_doubly_linked_list_iterator_from_object(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    size_t line,
    PtnArrayIterator *out
);
static PTN_UNUSED PtnArray *ptn_spl_doubly_linked_list_iterator_remove_index(
    PtnRuntime *runtime,
    PtnObject *object,
    size_t physical_index,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_file_info_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_file_info_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_file_info_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_file_object_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_file_object_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_file_object_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_directory_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_directory_iterator_new_for_class(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_directory_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_directory_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_regex_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_regex_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_call_internal(
    PtnRuntime *runtime,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_iterator_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_append_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_append_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_no_rewind_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_multiple_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_iterator_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_multiple_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_recursive_iterator_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_recursive_iterator_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_callback_filter_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_recursive_callback_filter_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_callback_filter_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_filter_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_infinite_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_limit_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_limit_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_datetime_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line,
    int malformed_returns_false
);
static PTN_UNUSED PtnValue ptn_datetime_zone_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_date_interval_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_bcmath_number_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_bcmath_number_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED int ptn_bcmath_number_cast_array(PtnValue value, PtnValue *array_out);
static PTN_UNUSED int ptn_bcmath_number_is_truthy(PtnValue value, int *truthy_out);
static PTN_UNUSED int ptn_bcmath_number_compare(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line,
    int *compared
);
static PTN_UNUSED int ptn_bcmath_number_binary_op(
    PtnRuntime *runtime,
    const char *operator,
    PtnValue left,
    PtnValue right,
    size_t line,
    PtnValue *result_out
);
static PTN_UNUSED int ptn_bcmath_number_inc_dec(
    PtnRuntime *runtime,
    PtnValue value,
    int increment,
    size_t line,
    PtnValue *result_out
);
static PTN_UNUSED void ptn_bcmath_number_hydrate_unserialized(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
);
static PTN_UNUSED PtnValue ptn_phar_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_zip_archive_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_zip_archive_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED void ptn_zip_archive_run_destructor(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t line
);
static PTN_UNUSED PtnValue ptn_soap_client_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_soap_header_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_soap_var_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_soap_param_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_soap_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_break_iterator_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_plain_object_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_xmlwriter_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_uri_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_break_iterator_clone(PtnRuntime *runtime, PtnValue source, size_t line);
static PTN_UNUSED PtnValue ptn_dom_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_xml_reader_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_xml_parser_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_simplexml_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_uri_whatwg_url_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_uri_url_validation_error_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_uri_clone(PtnRuntime *runtime, PtnValue source, size_t line);
static PTN_UNUSED PtnValue ptn_datetime_clone(PtnRuntime *runtime, PtnValue source, size_t line);
static PTN_UNUSED PtnValue ptn_datetime_zone_clone(PtnRuntime *runtime, PtnValue source, size_t line);
static PTN_UNUSED PtnValue ptn_datetime_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_datetime_zone_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_date_interval_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_date_period_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_internal_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_break_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_date_formatter_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_calendar_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_timezone_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_message_formatter_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_list_formatter_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_number_formatter_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_collator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_spoofchecker_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_uconverter_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_xmlwriter_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_php_token_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_dom_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_xml_reader_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_simplexml_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_simplexml_clone(PtnRuntime *runtime, PtnValue source, size_t line);
static PTN_UNUSED int ptn_simplexml_is_truthy(PtnValue value, int *truthy_out);
static PTN_UNUSED int ptn_simplexml_numeric_value(PtnValue value, PtnNumber *number_out);
static PTN_UNUSED int ptn_simplexml_property_is_set(PtnValue receiver, const char *property, int *isset_out);
static PtnValue ptn_internal_simplexml_load_string(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_simplexml_load_file(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PTN_UNUSED PtnValue ptn_uri_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_uri_whatwg_url_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_uri_url_validation_error_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_internal_class_static_call_method(
    PtnRuntime *runtime,
    const char *class_name,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
#endif

static PTN_UNUSED int ptn_ini_precision_value(
    const char *configured,
    int default_value,
    int max_value
) {
    if (configured != NULL && configured[0] != '\0') {
        char *end = NULL;
        errno = 0;
        long parsed = strtol(configured, &end, 10);
        if (errno == 0 && end != configured && *end == '\0' && parsed >= -1 &&
            parsed <= max_value) {
            return (int)parsed;
        }
    }
    return default_value;
}

static PTN_UNUSED int ptn_default_float_precision(void) {
    static int initialized = 0;
    static int precision = PTN_DEFAULT_PRECISION;
    if (!initialized) {
        precision = ptn_ini_precision_value(
            getenv("PTN_PHP_PRECISION"),
            PTN_DEFAULT_PRECISION,
            PTN_MAX_FLOAT_FORMAT_PRECISION
        );
        initialized = 1;
    }
    return precision;
}

static PTN_UNUSED int ptn_default_serialize_precision(void) {
    static int initialized = 0;
    static int precision = PTN_DEFAULT_SERIALIZE_PRECISION;
    if (!initialized) {
        precision = ptn_ini_precision_value(
            getenv("PTN_PHP_SERIALIZE_PRECISION"),
            PTN_DEFAULT_SERIALIZE_PRECISION,
            PTN_MAX_FLOAT_FORMAT_PRECISION
        );
        initialized = 1;
    }
    return precision;
}

static PTN_UNUSED int ptn_runtime_float_precision(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    return root == NULL ? ptn_default_float_precision() : root->precision;
}

static PTN_UNUSED int ptn_runtime_serialize_precision(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    return root == NULL ? ptn_default_serialize_precision() : root->serialize_precision;
}

static PTN_UNUSED void ptn_normalize_scalar_float_exponent(char *buffer) {
    for (char *cursor = buffer; *cursor != '\0'; cursor++) {
        if (*cursor == 'e' || *cursor == 'E') {
            *cursor = 'E';
            cursor++;
            if (*cursor == '+' || *cursor == '-') {
                cursor++;
            }
            while (*cursor == '0' && isdigit((unsigned char)cursor[1])) {
                memmove(cursor, cursor + 1, strlen(cursor));
            }
            return;
        }
    }
}

static PTN_UNUSED void ptn_scalar_float_ensure_exponent_decimal(char *buffer) {
    for (char *cursor = buffer; *cursor != '\0'; cursor++) {
        if (*cursor == '.') {
            return;
        }
        if (*cursor == 'E') {
            size_t tail_len = strlen(cursor);
            memmove(cursor + 2, cursor, tail_len + 1);
            cursor[0] = '.';
            cursor[1] = '0';
            return;
        }
    }
}

static PTN_UNUSED int ptn_same_scalar_double(double left, double right) {
    return memcmp(&left, &right, sizeof(double)) == 0;
}

static PTN_UNUSED void ptn_format_scalar_shortest_float(double value, char *buffer, size_t buffer_size) {
    for (int precision = 1; precision <= 17; precision++) {
        char candidate[64];
        char *end = NULL;
        double reparsed;
        snprintf(candidate, sizeof(candidate), "%.*g", precision, value);
        ptn_normalize_scalar_float_exponent(candidate);
        ptn_scalar_float_ensure_exponent_decimal(candidate);
        errno = 0;
        reparsed = strtod(candidate, &end);
        if (errno == 0 && end != NULL && *end == '\0' && ptn_same_scalar_double(reparsed, value)) {
            int written = snprintf(buffer, buffer_size, "%s", candidate);
            if (written < 0 || (size_t)written >= buffer_size) {
                ptn_abort_out_of_memory();
            }
            return;
        }
    }

    int written = snprintf(buffer, buffer_size, "%.17g", value);
    if (written < 0 || (size_t)written >= buffer_size) {
        ptn_abort_out_of_memory();
    }
    ptn_normalize_scalar_float_exponent(buffer);
    ptn_scalar_float_ensure_exponent_decimal(buffer);
}

static PTN_UNUSED int ptn_formatted_float_has_decimal_or_exponent(const char *buffer) {
    for (const char *cursor = buffer; *cursor != '\0'; cursor++) {
        if (*cursor == '.' || *cursor == 'E' || *cursor == 'e') {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED void ptn_format_scalar_float_with_precision(
    double value,
    int precision,
    char *buffer,
    size_t buffer_size
) {
    int written;
    if (isnan(value)) {
        written = snprintf(buffer, buffer_size, "NAN");
    } else if (isinf(value)) {
        written = snprintf(buffer, buffer_size, signbit(value) ? "-INF" : "INF");
    } else if (precision < 0) {
        ptn_format_scalar_shortest_float(value, buffer, buffer_size);
        return;
    } else {
        written = snprintf(buffer, buffer_size, "%.*g", precision, value);
    }
    if (written < 0 || (size_t)written >= buffer_size) {
        ptn_abort_out_of_memory();
    }
    ptn_normalize_scalar_float_exponent(buffer);
    ptn_scalar_float_ensure_exponent_decimal(buffer);
}

static PTN_UNUSED void ptn_format_scalar_float(double value, char *buffer, size_t buffer_size) {
    ptn_format_scalar_float_with_precision(value, ptn_default_float_precision(), buffer, buffer_size);
}

static PTN_UNUSED void ptn_format_runtime_scalar_float(
    PtnRuntime *runtime,
    double value,
    char *buffer,
    size_t buffer_size
) {
    ptn_format_scalar_float_with_precision(
        value,
        ptn_runtime_float_precision(runtime),
        buffer,
        buffer_size
    );
}

static PTN_UNUSED void ptn_format_serialize_float_with_precision(
    double value,
    int precision,
    char *buffer,
    size_t buffer_size
) {
    ptn_format_scalar_float_with_precision(value, precision, buffer, buffer_size);
    if (precision == 0 &&
        isfinite(value) &&
        !ptn_formatted_float_has_decimal_or_exponent(buffer)) {
        size_t len = strlen(buffer);
        if (buffer_size < 6 || len > buffer_size - 6) {
            ptn_abort_out_of_memory();
        }
        memcpy(buffer + len, ".0E+0", 6);
    }
}

static PTN_UNUSED void ptn_format_runtime_serialize_float(
    PtnRuntime *runtime,
    double value,
    char *buffer,
    size_t buffer_size
) {
    ptn_format_serialize_float_with_precision(
        value,
        ptn_runtime_serialize_precision(runtime),
        buffer,
        buffer_size
    );
}

static PTN_UNUSED void ptn_format_runtime_var_export_float(
    PtnRuntime *runtime,
    double value,
    char *buffer,
    size_t buffer_size
) {
    ptn_format_scalar_float_with_precision(
        value,
        ptn_runtime_serialize_precision(runtime),
        buffer,
        buffer_size
    );
    if (isfinite(value) && !ptn_formatted_float_has_decimal_or_exponent(buffer)) {
        size_t len = strlen(buffer);
        if (buffer_size < 3 || len > buffer_size - 3) {
            ptn_abort_out_of_memory();
        }
        buffer[len] = '.';
        buffer[len + 1] = '0';
        buffer[len + 2] = '\0';
    }
}

static PTN_UNUSED PtnValue ptn_null(void) {
    PtnValue value;
    value.type = PTN_NULL;
    value.owned = 0;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    return value;
}

static PTN_UNUSED PtnValue ptn_missing(void) {
    PtnValue value;
    value.type = PTN_NULL;
    value.owned = -1;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    return value;
}

static PTN_UNUSED PtnValue ptn_nullsafe_short_circuit(void) {
    PtnValue value;
    value.type = PTN_NULL;
    value.owned = -2;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    return value;
}

static PTN_UNUSED int ptn_value_is_missing(PtnValue value) {
    return value.type == PTN_NULL && value.owned == -1;
}

static PTN_UNUSED int ptn_value_is_nullsafe_short_circuit(PtnValue value) {
    while (value.type == PTN_REFERENCE) {
        value = value.as.reference->value;
    }
    return value.type == PTN_NULL && value.owned == -2;
}

static PTN_UNUSED int ptn_value_is_return_reference_fallback(PtnValue value) {
    return value.type != PTN_REFERENCE && value.by_ref_return_fallback;
}

static PTN_UNUSED PtnValue ptn_value_mark_return_reference_fallback(PtnValue value) {
    if (value.type != PTN_REFERENCE) {
        value.by_ref_return_fallback = 1;
    }
    return value;
}

static PTN_UNUSED int ptn_value_is_by_ref_argument_source(PtnValue value) {
    return value.type == PTN_REFERENCE && !value.by_ref_argument_source_disabled;
}

static PTN_UNUSED PtnValue ptn_value_disable_by_ref_argument_source(PtnValue value) {
    if (value.type == PTN_REFERENCE) {
        value.by_ref_argument_source_disabled = 1;
    }
    return value;
}

static PTN_UNUSED PtnValue ptn_bool(int boolean) {
    PtnValue value;
    value.type = PTN_BOOL;
    value.owned = 0;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    value.as.boolean = boolean ? 1 : 0;
    return value;
}

static PTN_UNUSED PtnValue ptn_int(int64_t integer) {
    PtnValue value;
    value.type = PTN_INT;
    value.owned = 0;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    value.as.integer = integer;
    return value;
}

static PTN_UNUSED PtnValue ptn_float(double floating) {
    PtnValue value;
    value.type = PTN_FLOAT;
    value.owned = 0;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    value.as.floating = floating;
    return value;
}

static PTN_UNUSED PtnStringPayload *ptn_string_payload_from_owned(char *string, size_t len) {
    PtnStringPayload *payload = malloc(sizeof(PtnStringPayload));
    if (payload == NULL) {
        free(string);
        ptn_abort_out_of_memory();
    }
    payload->refcount = 1;
    payload->len = len;
    payload->data = (unsigned char *)string;
    payload->data[len] = '\0';
    payload->interned = 0;
    ptn_cow_debug_note_string_alloc();
    return payload;
}

static PTN_UNUSED void ptn_string_payload_retain(PtnStringPayload *payload) {
    if (payload == NULL) {
        return;
    }
    ptn_cow_debug_assert_string_refcount(&payload->refcount, "retain");
    if (payload->refcount == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    ptn_cow_debug_note_string_clone();
    ptn_cow_debug_note_string_retain();
    payload->refcount++;
}

static PTN_UNUSED void ptn_string_payload_release(PtnStringPayload *payload) {
    if (payload == NULL) {
        return;
    }
    if (payload->refcount == 0) {
        return;
    }
    ptn_cow_debug_assert_string_refcount(&payload->refcount, "release");
    ptn_cow_debug_note_string_release();
    payload->refcount--;
    if (payload->refcount != 0) {
        return;
    }
    ptn_cow_debug_note_string_free();
    free(payload->data);
    free(payload);
}

static PTN_UNUSED void ptn_string_value_refresh(PtnValue *value) {
    if (value == NULL || value->type != PTN_STRING || value->as.string.payload == NULL) {
        return;
    }
    value->as.string.data = value->as.string.payload->data;
    value->as.string.len = value->as.string.payload->len;
}

static PTN_UNUSED void ptn_string_value_resize(PtnValue *value, size_t new_len) {
    if (value == NULL ||
        value->type != PTN_STRING ||
        !value->owned ||
        value->as.string.payload == NULL ||
        value->as.string.payload->refcount != 1) {
        ptn_abort_out_of_memory();
    }
    if (new_len == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }

    PtnStringPayload *payload = value->as.string.payload;
    size_t old_len = payload->len;
    unsigned char *data = realloc(payload->data, new_len + 1);
    if (data == NULL) {
        ptn_abort_out_of_memory();
    }
    if (new_len > old_len) {
        memset(data + old_len, ' ', new_len - old_len);
    }
    data[new_len] = '\0';
    payload->data = data;
    payload->len = new_len;
    payload->interned = 0;
    ptn_string_value_refresh(value);
}

static PTN_UNUSED PtnValue ptn_string_literal(const char *string, size_t len) {
    PtnValue value;
    value.type = PTN_STRING;
    value.owned = 0;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    value.as.string.data = (const unsigned char *)string;
    value.as.string.len = len;
    value.as.string.payload = NULL;
    return value;
}

static PTN_UNUSED PtnValue ptn_string(const char *string) {
    return ptn_string_literal(string, strlen(string));
}

static PTN_UNUSED PtnValue ptn_owned_string_len(char *string, size_t len) {
    PtnStringPayload *payload = ptn_string_payload_from_owned(string, len);
    PtnValue value;
    value.type = PTN_STRING;
    value.owned = 1;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    value.as.string.data = payload->data;
    value.as.string.len = len;
    value.as.string.payload = payload;
    return value;
}

static PTN_UNUSED PtnValue ptn_owned_string(char *string) {
    return ptn_owned_string_len(string, strlen(string));
}

static PTN_UNUSED PtnValue ptn_array(PtnArray *array) {
    PtnValue value;
    value.type = PTN_ARRAY;
    value.owned = 1;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    value.as.array = array;
    return value;
}

static PTN_UNUSED PtnValue ptn_object(PtnObject *object) {
    PtnValue value;
    value.type = PTN_OBJECT;
    value.owned = 1;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    value.as.object = object;
    return value;
}

static PTN_UNUSED PtnValue ptn_closure(
    PtnRuntime *runtime,
    size_t function_index,
    const char *display_name,
    PtnFunctionMetadata metadata,
    int is_static,
    int uses_this
) {
    PtnClosure *closure = malloc(sizeof(PtnClosure));
    if (closure == NULL) {
        ptn_abort_out_of_memory();
    }
    closure->refcount = 1;
    closure->object_id = ptn_runtime_alloc_object_id(runtime);
    closure->gc_mark_epoch = 0;
    closure->lifecycle_runtime = ptn_runtime_root(runtime);
    closure->function_index = function_index;
    closure->display_name = display_name;
    closure->metadata = metadata;
    closure->scope_class_name = NULL;
    closure->called_class_name = NULL;
    closure->is_static = is_static;
    closure->uses_this = uses_this;
    closure->captures.items = NULL;
    closure->captures.len = 0;
    closure->captures.capacity = 0;
    closure->captures.index_slots = NULL;
    closure->captures.index_capacity = 0;
    closure->has_wrapped_callable = 0;
    closure->suppress_wrapped_callable_deprecation = 0;
    closure->wrapped_callable = ptn_null();
    closure->bound_scope_name = NULL;
    closure->origin_kind = PTN_CLOSURE_ORIGIN_ANONYMOUS;
    closure->origin_class_name = NULL;
    closure->origin_method_name = NULL;
    ptn_runtime_register_closure(runtime, closure);
    PtnValue value;
    value.type = PTN_CLOSURE;
    value.owned = 1;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    value.as.closure = closure;
    return value;
}

static PTN_UNUSED PtnValue ptn_exception_value(PtnException *exception) {
    PtnValue value;
    value.type = PTN_EXCEPTION;
    value.owned = 1;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    value.as.exception = exception;
    return value;
}

static PTN_UNUSED PtnValue ptn_exception_borrow(PtnException *exception) {
    PtnValue value = ptn_exception_value(exception);
    value.owned = 0;
    return value;
}

static PTN_UNUSED void ptn_exception_retain(PtnException *exception) {
    if (exception == NULL) {
        return;
    }
    if (exception->refcount == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    exception->refcount++;
}

static int64_t ptn_next_resource_id = 4;
static PtnResource *ptn_resource_registry_head = NULL;
static PtnResource *ptn_resource_registry_tail = NULL;

static PTN_UNUSED void ptn_resource_register(PtnResource *resource) {
    if (resource == NULL || resource->persistent) {
        return;
    }
    resource->registry_prev = ptn_resource_registry_tail;
    resource->registry_next = NULL;
    resource->manual_close_forbidden = 0;
    if (ptn_resource_registry_tail != NULL) {
        ptn_resource_registry_tail->registry_next = resource;
    } else {
        ptn_resource_registry_head = resource;
    }
    ptn_resource_registry_tail = resource;
}

static PTN_UNUSED void ptn_resource_unregister(PtnResource *resource) {
    if (resource == NULL || resource->persistent) {
        return;
    }
    if (resource->registry_prev != NULL) {
        resource->registry_prev->registry_next = resource->registry_next;
    } else if (ptn_resource_registry_head == resource) {
        ptn_resource_registry_head = resource->registry_next;
    }
    if (resource->registry_next != NULL) {
        resource->registry_next->registry_prev = resource->registry_prev;
    } else if (ptn_resource_registry_tail == resource) {
        ptn_resource_registry_tail = resource->registry_prev;
    }
    resource->registry_prev = NULL;
    resource->registry_next = NULL;
}

static PTN_UNUSED void ptn_resource_forbid_manual_close(PtnResource *resource) {
    if (resource != NULL) {
        resource->manual_close_forbidden = 1;
    }
}

static PTN_UNUSED int ptn_resource_manual_close_forbidden(PtnResource *resource) {
    return resource != NULL && resource->manual_close_forbidden;
}

static PTN_UNUSED PtnMemoryStream *ptn_memory_stream_new(size_t max_memory, int writable, int append) {
    PtnMemoryStream *stream = malloc(sizeof(PtnMemoryStream));
    if (stream == NULL) {
        ptn_abort_out_of_memory();
    }
    stream->data = NULL;
    stream->len = 0;
    stream->capacity = 0;
    stream->position = 0;
    stream->max_memory = max_memory;
    stream->writable = writable;
    stream->append = append;
    stream->spilled = 0;
    stream->eof = 0;
    stream->error = 0;
    return stream;
}

static PTN_UNUSED void ptn_memory_stream_free(PtnMemoryStream *stream) {
    if (stream == NULL) {
        return;
    }
    free(stream->data);
    free(stream);
}

static PTN_UNUSED int ptn_resource_is_open(PtnResource *resource) {
    if (resource == NULL) {
        return 0;
    }
    if (resource->closed) {
        return 0;
    }
    return resource->stream != NULL ||
        resource->directory != NULL ||
        resource->memory_stream != NULL ||
        strcmp(resource->type_name, "stream") != 0;
}

static PTN_UNUSED const char *ptn_resource_display_type(PtnResource *resource) {
    return ptn_resource_is_open(resource) ? resource->type_name : "Unknown";
}

static PTN_UNUSED const char *ptn_resource_curl_class_name(PtnResource *resource) {
    if (resource == NULL || !ptn_resource_is_open(resource) || resource->type_name == NULL) {
        return NULL;
    }
    if (strcmp(resource->type_name, "curl") == 0) {
        return "CurlHandle";
    }
    if (strcmp(resource->type_name, "curl_multi") == 0) {
        return "CurlMultiHandle";
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_resource_object_class_name(PtnResource *resource) {
    const char *curl_class_name = ptn_resource_curl_class_name(resource);
    if (curl_class_name != NULL) {
        return curl_class_name;
    }
    if (resource == NULL || !ptn_resource_is_open(resource) || resource->type_name == NULL) {
        return NULL;
    }
    if (strcmp(resource->type_name, "Socket") == 0) {
        return "Socket";
    }
    return NULL;
}

static PTN_UNUSED int ptn_resource_is_curl_handle(PtnResource *resource) {
    return ptn_resource_curl_class_name(resource) != NULL;
}

static PTN_UNUSED size_t ptn_resource_object_id(PtnResource *resource) {
    if (resource == NULL) {
        return 0;
    }
    if (resource->object_id != 0) {
        return resource->object_id;
    }
    if (resource->id <= 0) {
        return 0;
    }
    return (size_t)resource->id;
}

static PTN_UNUSED void ptn_resource_assign_object_id(PtnRuntime *runtime, PtnResource *resource) {
    if (resource == NULL || resource->object_id != 0) {
        return;
    }
    resource->object_id = ptn_runtime_alloc_object_id(runtime);
    resource->object_id_runtime = runtime;
}

static PTN_UNUSED int ptn_stream_resource_is_open(PtnResource *resource) {
    if (resource == NULL || resource->closed) {
        return 0;
    }
    if (resource->persistent && resource->id >= 1 && resource->id <= 3) {
        return 1;
    }
    return resource->stream != NULL || resource->memory_stream != NULL;
}

static PTN_UNUSED PtnResource *ptn_resource_new_stream(FILE *stream, const char *uri, const char *mode) {
    PtnResource *resource = malloc(sizeof(PtnResource));
    if (resource == NULL) {
        if (stream != NULL) {
            fclose(stream);
        }
        ptn_abort_out_of_memory();
    }
    if (ptn_next_resource_id == INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    resource->refcount = 1;
    resource->id = ptn_next_resource_id++;
    resource->type_name = "stream";
    resource->stream = stream;
    resource->directory = NULL;
    resource->stream_uri = uri == NULL ? NULL : ptn_duplicate_string(uri);
    resource->stream_mode = mode == NULL ? NULL : ptn_duplicate_string(mode);
    resource->stream_backend = PTN_STREAM_BACKEND_FILE;
    resource->memory_stream = NULL;
    resource->read_filters = NULL;
    resource->write_filters = NULL;
    resource->filtered_read_buffer = NULL;
    resource->filtered_read_buffer_len = 0;
    resource->filtered_read_buffer_offset = 0;
    resource->chunk_size = 8192;
    resource->close_hook = NULL;
    resource->close_hook_data = NULL;
    resource->close_hook_data_free = NULL;
    resource->persistent = 0;
    resource->closed = 0;
    resource->context_options = ptn_null();
    resource->context_params = ptn_null();
    resource->curl_options = ptn_null();
    ptn_resource_register(resource);
    resource->manual_close_forbidden = 0;
    resource->object_id = 0;
    resource->object_id_runtime = NULL;
    return resource;
}

static PTN_UNUSED PtnResource *ptn_resource_new_memory_stream(
    const char *uri,
    const char *mode,
    PtnStreamBackend backend,
    size_t max_memory,
    int writable,
    int append
) {
    PtnResource *resource = malloc(sizeof(PtnResource));
    if (resource == NULL) {
        ptn_abort_out_of_memory();
    }
    if (ptn_next_resource_id == INT64_MAX) {
        free(resource);
        ptn_abort_out_of_memory();
    }
    resource->refcount = 1;
    resource->id = ptn_next_resource_id++;
    resource->type_name = "stream";
    resource->stream = NULL;
    resource->directory = NULL;
    resource->stream_uri = uri == NULL ? NULL : ptn_duplicate_string(uri);
    resource->stream_mode = mode == NULL ? NULL : ptn_duplicate_string(mode);
    resource->stream_backend = backend;
    resource->memory_stream = ptn_memory_stream_new(max_memory, writable, append);
    resource->read_filters = NULL;
    resource->write_filters = NULL;
    resource->filtered_read_buffer = NULL;
    resource->filtered_read_buffer_len = 0;
    resource->filtered_read_buffer_offset = 0;
    resource->chunk_size = 8192;
    resource->close_hook = NULL;
    resource->close_hook_data = NULL;
    resource->close_hook_data_free = NULL;
    resource->persistent = 0;
    resource->closed = 0;
    resource->context_options = ptn_null();
    resource->context_params = ptn_null();
    resource->curl_options = ptn_null();
    ptn_resource_register(resource);
    resource->manual_close_forbidden = 0;
    resource->object_id = 0;
    resource->object_id_runtime = NULL;
    return resource;
}

static PTN_UNUSED PtnResource *ptn_resource_new_directory(void *directory, const char *uri) {
    PtnResource *resource = malloc(sizeof(PtnResource));
    if (resource == NULL) {
#if !defined(_WIN32)
        if (directory != NULL) {
            closedir((DIR *)directory);
        }
#endif
        ptn_abort_out_of_memory();
    }
    if (ptn_next_resource_id == INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    resource->refcount = 1;
    resource->id = ptn_next_resource_id++;
    resource->type_name = "stream";
    resource->stream = NULL;
    resource->directory = directory;
    resource->stream_uri = uri == NULL ? NULL : ptn_duplicate_string(uri);
    resource->stream_mode = ptn_duplicate_string("r");
    resource->stream_backend = PTN_STREAM_BACKEND_FILE;
    resource->memory_stream = NULL;
    resource->read_filters = NULL;
    resource->write_filters = NULL;
    resource->filtered_read_buffer = NULL;
    resource->filtered_read_buffer_len = 0;
    resource->filtered_read_buffer_offset = 0;
    resource->chunk_size = 8192;
    resource->close_hook = NULL;
    resource->close_hook_data = NULL;
    resource->close_hook_data_free = NULL;
    resource->persistent = 0;
    resource->closed = 0;
    resource->context_options = ptn_null();
    resource->context_params = ptn_null();
    resource->curl_options = ptn_null();
    ptn_resource_register(resource);
    resource->manual_close_forbidden = 0;
    resource->object_id = 0;
    resource->object_id_runtime = NULL;
    return resource;
}

static PTN_UNUSED PtnResource *ptn_resource_new_named(const char *type_name) {
    PtnResource *resource = malloc(sizeof(PtnResource));
    if (resource == NULL) {
        ptn_abort_out_of_memory();
    }
    if (ptn_next_resource_id == INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    resource->refcount = 1;
    resource->id = ptn_next_resource_id++;
    resource->type_name = type_name;
    resource->stream = NULL;
    resource->directory = NULL;
    resource->stream_uri = NULL;
    resource->stream_mode = NULL;
    resource->stream_backend = PTN_STREAM_BACKEND_FILE;
    resource->memory_stream = NULL;
    resource->read_filters = NULL;
    resource->write_filters = NULL;
    resource->filtered_read_buffer = NULL;
    resource->filtered_read_buffer_len = 0;
    resource->filtered_read_buffer_offset = 0;
    resource->chunk_size = 8192;
    resource->close_hook = NULL;
    resource->close_hook_data = NULL;
    resource->close_hook_data_free = NULL;
    resource->persistent = 0;
    resource->closed = 0;
    resource->context_options = ptn_null();
    resource->context_params = ptn_null();
    resource->curl_options = ptn_null();
    resource->manual_close_forbidden = 0;
    resource->object_id = 0;
    resource->object_id_runtime = NULL;
    ptn_resource_register(resource);
    return resource;
}

static PTN_UNUSED int ptn_memory_stream_reserve(PtnMemoryStream *stream, size_t required) {
    if (required <= stream->capacity) {
        return 1;
    }
    size_t new_capacity = stream->capacity == 0 ? 128 : stream->capacity;
    while (new_capacity < required) {
        if (new_capacity > SIZE_MAX / 2) {
            return 0;
        }
        new_capacity *= 2;
    }
    unsigned char *new_data = realloc(stream->data, new_capacity);
    if (new_data == NULL) {
        return 0;
    }
    stream->data = new_data;
    stream->capacity = new_capacity;
    return 1;
}

static PTN_UNUSED void ptn_memory_stream_note_size(PtnResource *resource, PtnMemoryStream *stream) {
    if (
        resource->stream_backend == PTN_STREAM_BACKEND_TEMP &&
        stream->max_memory != SIZE_MAX &&
        stream->len > stream->max_memory
    ) {
        stream->spilled = 1;
    }
}

static PTN_UNUSED size_t ptn_stream_write_bytes(PtnResource *resource, const void *data, size_t len) {
    if (resource == NULL) {
        return 0;
    }
    if (resource->memory_stream == NULL) {
        if (resource->stream == NULL) {
            errno = EBADF;
            return 0;
        }
        size_t written = fwrite(data, 1, len, resource->stream);
        if (written > 0) {
            (void)fflush(resource->stream);
        }
        return written;
    }

    PtnMemoryStream *stream = resource->memory_stream;
    if (!stream->writable) {
        stream->error = 1;
        errno = EBADF;
        return 0;
    }
    if (stream->append) {
        stream->position = stream->len;
    }
    if (stream->position > SIZE_MAX - len) {
        ptn_abort_out_of_memory();
    }
    size_t end = stream->position + len;
    if (!ptn_memory_stream_reserve(stream, end)) {
        ptn_abort_out_of_memory();
    }
    if (stream->position > stream->len) {
        memset(stream->data + stream->len, 0, stream->position - stream->len);
    }
    if (len != 0) {
        memcpy(stream->data + stream->position, data, len);
    }
    stream->position = end;
    if (end > stream->len) {
        stream->len = end;
        ptn_memory_stream_note_size(resource, stream);
    }
    stream->error = 0;
    return len;
}

static PTN_UNUSED int ptn_stream_errno_would_block(int error) {
#if defined(EWOULDBLOCK) && defined(EAGAIN)
    return error == EWOULDBLOCK || error == EAGAIN;
#elif defined(EWOULDBLOCK)
    return error == EWOULDBLOCK;
#elif defined(EAGAIN)
    return error == EAGAIN;
#else
    (void)error;
    return 0;
#endif
}

static PTN_UNUSED size_t ptn_stream_read_bytes(PtnResource *resource, void *buffer, size_t len) {
    if (resource == NULL) {
        return 0;
    }
    if (resource->memory_stream == NULL) {
        if (resource->stream == NULL) {
            errno = EBADF;
            return 0;
        }
        size_t read_len = fread(buffer, 1, len, resource->stream);
        if (read_len == 0 && feof(resource->stream) && len != 0) {
            clearerr(resource->stream);
            read_len = fread(buffer, 1, len, resource->stream);
        }
        if (read_len == 0 && ferror(resource->stream) && ptn_stream_errno_would_block(errno)) {
            clearerr(resource->stream);
        }
        return read_len;
    }

    PtnMemoryStream *stream = resource->memory_stream;
    if (len == 0) {
        return 0;
    }
    if (stream->position >= stream->len) {
        stream->eof = 1;
        return 0;
    }
    size_t available = stream->len - stream->position;
    size_t read_len = available < len ? available : len;
    memcpy(buffer, stream->data + stream->position, read_len);
    stream->position += read_len;
    stream->eof = read_len < len;
    stream->error = 0;
    return read_len;
}

static PTN_UNUSED int ptn_stream_get_byte(PtnResource *resource) {
    if (resource == NULL) {
        return EOF;
    }
    if (resource->memory_stream == NULL) {
        if (resource->stream == NULL) {
            errno = EBADF;
            return EOF;
        }
        int byte = fgetc(resource->stream);
        if (byte == EOF && ferror(resource->stream) && ptn_stream_errno_would_block(errno)) {
            clearerr(resource->stream);
        }
        return byte;
    }
    unsigned char byte = 0;
    return ptn_stream_read_bytes(resource, &byte, 1) == 1 ? (int)byte : EOF;
}

static PTN_UNUSED int ptn_stream_unget_byte(PtnResource *resource, int byte) {
    if (resource == NULL) {
        return EOF;
    }
    if (resource->memory_stream == NULL) {
        if (resource->stream == NULL) {
            errno = EBADF;
            return EOF;
        }
        return ungetc(byte, resource->stream);
    }
    PtnMemoryStream *stream = resource->memory_stream;
    if (stream->position == 0 || byte == EOF) {
        return EOF;
    }
    stream->position--;
    stream->eof = 0;
    stream->error = 0;
    return byte;
}

static PTN_UNUSED int ptn_stream_seek(PtnResource *resource, int64_t offset, int whence) {
    if (resource == NULL) {
        return -1;
    }
    if (resource->memory_stream == NULL) {
        if (resource->stream == NULL) {
            errno = EBADF;
            return -1;
        }
        return fseek(resource->stream, (long)offset, whence);
    }
    PtnMemoryStream *stream = resource->memory_stream;
    size_t base_size = 0;
    if (whence == SEEK_SET) {
        base_size = 0;
    } else if (whence == SEEK_CUR) {
        base_size = stream->position;
    } else if (whence == SEEK_END) {
        base_size = stream->len;
    } else {
        return -1;
    }
    if (base_size > (size_t)INT64_MAX) {
        return -1;
    }
    int64_t base = (int64_t)base_size;
    if (offset < 0) {
        if (offset == INT64_MIN || base < -offset) {
            return -1;
        }
    } else if (base > INT64_MAX - offset) {
        return -1;
    }
    int64_t target = base + offset;
    if (target < 0) {
        return -1;
    }
    stream->position = (size_t)target;
    stream->eof = 0;
    stream->error = 0;
    return 0;
}

static PTN_UNUSED int64_t ptn_stream_tell(PtnResource *resource) {
    if (resource == NULL) {
        return -1;
    }
    if (resource->memory_stream == NULL) {
        if (resource->stream == NULL) {
            errno = EBADF;
            return -1;
        }
        long position = ftell(resource->stream);
        return position < 0 ? -1 : (int64_t)position;
    }
    PtnMemoryStream *stream = resource->memory_stream;
    if (stream->position > (size_t)INT64_MAX) {
        return -1;
    }
    return (int64_t)stream->position;
}

static PTN_UNUSED int ptn_stream_flush(PtnResource *resource) {
    if (resource == NULL) {
        return -1;
    }
    if (resource->memory_stream == NULL) {
        if (resource->stream == NULL) {
            errno = EBADF;
            return -1;
        }
        return fflush(resource->stream);
    }
    resource->memory_stream->error = 0;
    return 0;
}

static PTN_UNUSED int ptn_stream_eof(PtnResource *resource) {
    if (resource == NULL) {
        return 1;
    }
    if (resource->memory_stream == NULL) {
        if (resource->stream == NULL) {
            return 1;
        }
        return feof(resource->stream) != 0;
    }
    return resource->memory_stream->eof != 0;
}

static PTN_UNUSED int ptn_stream_error(PtnResource *resource) {
    if (resource == NULL) {
        return 1;
    }
    if (resource->memory_stream == NULL) {
        if (resource->stream == NULL) {
            return 1;
        }
        return ferror(resource->stream) != 0;
    }
    return resource->memory_stream->error != 0;
}

static PTN_UNUSED void ptn_stream_clear_error(PtnResource *resource) {
    if (resource == NULL) {
        return;
    }
    if (resource->memory_stream == NULL) {
        if (resource->stream == NULL) {
            return;
        }
        clearerr(resource->stream);
        return;
    }
    resource->memory_stream->eof = 0;
    resource->memory_stream->error = 0;
}

static PTN_UNUSED int ptn_stream_truncate(PtnResource *resource, int64_t size) {
    if (resource == NULL) {
        return 0;
    }
    if (resource->memory_stream == NULL) {
        if (resource->stream == NULL) {
            errno = EBADF;
            return 0;
        }
        int descriptor = -1;
#if defined(_WIN32)
        descriptor = _fileno(resource->stream);
#else
        descriptor = fileno(resource->stream);
#endif
        if (descriptor < 0) {
            return 0;
        }
#if defined(_WIN32)
        return _chsize_s(descriptor, size) == 0;
#else
        return ftruncate(descriptor, (off_t)size) == 0;
#endif
    }

    PtnMemoryStream *stream = resource->memory_stream;
    if (!stream->writable) {
        stream->error = 1;
        errno = EBADF;
        return 0;
    }
    size_t new_len = (size_t)size;
    if (!ptn_memory_stream_reserve(stream, new_len)) {
        ptn_abort_out_of_memory();
    }
    if (new_len > stream->len) {
        memset(stream->data + stream->len, 0, new_len - stream->len);
    }
    stream->len = new_len;
    ptn_memory_stream_note_size(resource, stream);
    stream->eof = 0;
    stream->error = 0;
    return 1;
}

static PTN_UNUSED int ptn_stream_user_filter_has_method(PtnStreamFilter *filter, const char *method_name) {
    if (filter == NULL ||
        !filter->has_user_filter_object ||
        filter->user_filter_runtime == NULL ||
        filter->user_filter_runtime->declared_method_metadata == NULL) {
        return 0;
    }
    PtnValue object = filter->user_filter_object;
    if (object.type != PTN_OBJECT || object.as.object == NULL || object.as.object->class_name == NULL) {
        return 0;
    }
    return filter->user_filter_runtime->declared_method_metadata(
        object.as.object->class_name,
        method_name
    ).found;
}

static PTN_UNUSED void ptn_stream_user_filter_call_on_close(PtnStreamFilter *filter) {
    if (filter == NULL ||
        !ptn_stream_user_filter_has_method(filter, "onClose") ||
        filter->user_filter_runtime->method_dispatch == NULL) {
        return;
    }
    PtnValue result = filter->user_filter_runtime->method_dispatch(
        filter->user_filter_runtime,
        filter->user_filter_object,
        "onClose",
        0,
        NULL,
        filter->user_filter_line
    );
    ptn_value_destroy(&result);
}

static PTN_UNUSED void ptn_stream_filter_cleanup_user_object(PtnStreamFilter *filter) {
    if (filter == NULL || !filter->has_user_filter_object) {
        return;
    }
    ptn_stream_user_filter_call_on_close(filter);
    ptn_value_destroy(&filter->user_filter_object);
    filter->user_filter_object = ptn_null();
    filter->user_filter_runtime = NULL;
    filter->has_user_filter_object = 0;
}

static PTN_UNUSED void ptn_stream_filter_chain_free(PtnStreamFilter *filter) {
    while (filter != NULL) {
        PtnStreamFilter *next = filter->next;
        ptn_stream_filter_cleanup_user_object(filter);
        free(filter->name);
        free(filter->filter_line_break);
        free(filter);
        filter = next;
    }
}

static PTN_UNUSED void ptn_stream_filter_chain_flush_closing(PtnStreamFilter *filter) {
    if (ptn_stream_filter_chain_flush_closing_hook == NULL) {
        return;
    }
    ptn_stream_filter_chain_flush_closing_hook(filter);
}

static PTN_UNUSED void ptn_resource_retain(PtnResource *resource) {
    if (resource == NULL) {
        return;
    }
    if (resource->persistent) {
        return;
    }
    if (resource->refcount == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    resource->refcount++;
}

static PTN_UNUSED void ptn_resource_close(PtnResource *resource) {
    if (resource == NULL) {
        return;
    }
    if (resource->closed &&
        resource->stream == NULL &&
        resource->memory_stream == NULL &&
        resource->directory == NULL) {
        return;
    }
    resource->closed = 1;
    if (resource->persistent) {
        ptn_stream_filter_chain_flush_closing(resource->read_filters);
        ptn_stream_filter_chain_flush_closing(resource->write_filters);
        ptn_stream_filter_chain_free(resource->read_filters);
        ptn_stream_filter_chain_free(resource->write_filters);
        resource->read_filters = NULL;
        resource->write_filters = NULL;
        resource->stream = NULL;
        resource->memory_stream = NULL;
#if !defined(_WIN32)
        resource->directory = NULL;
#endif
        return;
    }
    if (resource->close_hook != NULL) {
        PtnResourceCloseHook close_hook = resource->close_hook;
        void *close_hook_data = resource->close_hook_data;
        PtnResourceHookDataFree close_hook_data_free = resource->close_hook_data_free;
        resource->close_hook = NULL;
        resource->close_hook_data = NULL;
        resource->close_hook_data_free = NULL;
        close_hook(resource, close_hook_data);
        if (close_hook_data_free != NULL) {
            close_hook_data_free(close_hook_data);
        }
    }
    ptn_stream_filter_chain_flush_closing(resource->read_filters);
    ptn_stream_filter_chain_flush_closing(resource->write_filters);
    ptn_stream_filter_chain_free(resource->read_filters);
    ptn_stream_filter_chain_free(resource->write_filters);
    resource->read_filters = NULL;
    resource->write_filters = NULL;
    if (resource->stream != NULL) {
        if (resource->stream_backend != PTN_STREAM_BACKEND_OUTPUT) {
            fclose(resource->stream);
        }
        resource->stream = NULL;
    }
    if (resource->memory_stream != NULL) {
        ptn_memory_stream_free(resource->memory_stream);
        resource->memory_stream = NULL;
    }
#if !defined(_WIN32)
    if (resource->directory != NULL) {
        closedir((DIR *)resource->directory);
        resource->directory = NULL;
    }
#endif
}

static PTN_UNUSED void ptn_resource_release(PtnResource *resource) {
    if (resource == NULL) {
        return;
    }
    if (resource->persistent) {
        return;
    }
    if (resource->refcount == 0) {
        return;
    }
    resource->refcount--;
    if (resource->refcount != 0) {
        return;
    }
    ptn_resource_unregister(resource);
    ptn_resource_close(resource);
    ptn_stream_filter_chain_free(resource->read_filters);
    ptn_stream_filter_chain_free(resource->write_filters);
    free(resource->filtered_read_buffer);
    free(resource->stream_uri);
    free(resource->stream_mode);
    ptn_value_destroy(&resource->context_options);
    ptn_value_destroy(&resource->context_params);
    ptn_value_destroy(&resource->curl_options);
    if (resource->object_id != 0) {
        ptn_runtime_release_object_id(resource->object_id_runtime, resource->object_id);
        resource->object_id = 0;
        resource->object_id_runtime = NULL;
    }
    free(resource);
}

static PTN_UNUSED PtnValue ptn_resource(PtnResource *resource) {
    PtnValue value;
    value.type = PTN_RESOURCE;
    value.owned = 1;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    value.as.resource = resource;
    return value;
}

static PTN_UNUSED PtnResource *ptn_standard_stream_resource_ptr(int64_t id) {
    static PtnResource stdin_resource = {
        SIZE_MAX,
        1,
        "stream",
        NULL,
        NULL,
        NULL,
        NULL,
        PTN_STREAM_BACKEND_FILE,
        NULL,
        NULL,
        NULL,
        NULL,
        0,
        0,
        8192,
        NULL,
        NULL,
        NULL,
        1,
        0,
        { PTN_NULL, 0, 0, 0, 0, { 0 } },
        { PTN_NULL, 0, 0, 0, 0, { 0 } }
    };
    static PtnResource stdout_resource = {
        SIZE_MAX,
        2,
        "stream",
        NULL,
        NULL,
        NULL,
        NULL,
        PTN_STREAM_BACKEND_FILE,
        NULL,
        NULL,
        NULL,
        NULL,
        0,
        0,
        8192,
        NULL,
        NULL,
        NULL,
        1,
        0,
        { PTN_NULL, 0, 0, 0, 0, { 0 } },
        { PTN_NULL, 0, 0, 0, 0, { 0 } }
    };
    static PtnResource stderr_resource = {
        SIZE_MAX,
        3,
        "stream",
        NULL,
        NULL,
        NULL,
        NULL,
        PTN_STREAM_BACKEND_FILE,
        NULL,
        NULL,
        NULL,
        NULL,
        0,
        0,
        8192,
        NULL,
        NULL,
        NULL,
        1,
        0,
        { PTN_NULL, 0, 0, 0, 0, { 0 } },
        { PTN_NULL, 0, 0, 0, 0, { 0 } }
    };
    PtnResource *resource = &stdin_resource;
    if (id == 2) {
        resource = &stdout_resource;
    } else if (id == 3) {
        resource = &stderr_resource;
    }
    return resource;
}

static PTN_UNUSED PtnValue ptn_standard_stream_resource_value(int64_t id) {
    PtnResource *resource = ptn_standard_stream_resource_ptr(id);
    resource->closed = 0;
    resource->stream = id == 1 ? stdin : (id == 2 ? stdout : stderr);
    resource->stream_uri = id == 1 ? "php://stdin" : (id == 2 ? "php://stdout" : "php://stderr");
    resource->stream_mode = id == 1 ? "r" : "w";
    PtnValue value = ptn_resource(resource);
    value.owned = 0;
    return value;
}

static PTN_UNUSED void ptn_standard_streams_shutdown(void) {
    ptn_resource_close(ptn_standard_stream_resource_ptr(1));
    ptn_resource_close(ptn_standard_stream_resource_ptr(2));
    ptn_resource_close(ptn_standard_stream_resource_ptr(3));
}

static PTN_UNUSED PtnValue ptn_reference_value(PtnReference *reference) {
    PtnValue value;
    value.type = PTN_REFERENCE;
    value.owned = 1;
    value.by_ref_return_fallback = 0;
    value.by_ref_argument_source_disabled = 0;
    value.from_string_offset = 0;
    value.as.reference = reference;
    return value;
}

static PTN_UNUSED PtnLookupResult ptn_lookup_missing(void) {
    PtnLookupResult result;
    result.exists = 0;
    result.value = ptn_null();
    return result;
}

static PTN_UNUSED PtnLookupResult ptn_lookup_found(PtnValue value) {
    PtnLookupResult result;
    result.exists = 1;
    result.value = value;
    return result;
}

static void ptn_abort_out_of_memory(void) {
    fputs("Fatal error: out of memory\n", stderr);
    exit(1);
}

static PTN_UNUSED void ptn_cow_debug_abort(const char *message) {
    fputs("Fatal error: COW debug assertion failed: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    exit(255);
}

static PTN_UNUSED void ptn_cow_debug_abort_counter(const char *name, size_t actual, int64_t expected) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "%s expected %lld, got %zu",
        name,
        (long long)expected,
        actual
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_cow_debug_abort(message);
}

static PTN_UNUSED void ptn_cow_debug_increment(size_t *counter) {
    if (*counter == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    (*counter)++;
}

static PTN_UNUSED void ptn_cow_debug_note_string_alloc(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.string_allocs);
}

static PTN_UNUSED void ptn_cow_debug_note_string_free(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.string_frees);
}

static PTN_UNUSED void ptn_cow_debug_note_string_clone(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.string_clones);
}

static PTN_UNUSED void ptn_cow_debug_note_string_retain(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.string_retain);
}

static PTN_UNUSED void ptn_cow_debug_note_string_release(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.string_release);
}

static PTN_UNUSED void ptn_cow_debug_note_string_detach(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.string_detaches);
}

static PTN_UNUSED void ptn_cow_debug_note_array_alloc(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_allocs);
}

static PTN_UNUSED void ptn_cow_debug_note_array_retain(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_retain);
}

static PTN_UNUSED void ptn_cow_debug_note_array_release(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_release);
}

static PTN_UNUSED void ptn_cow_debug_note_array_free(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_frees);
}

static PTN_UNUSED void ptn_cow_debug_note_array_clone(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_clones);
}

static PTN_UNUSED void ptn_cow_debug_note_array_detach(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_detaches);
}

static PTN_UNUSED void ptn_cow_debug_note_array_detach_skip(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_detach_skips);
}

static PTN_UNUSED void ptn_cow_debug_assert_array_refcount(PtnArray *array, const char *operation) {
    if (array == NULL) {
        return;
    }
    if (array->refcount == 0) {
        char message[128];
        int written = snprintf(message, sizeof(message), "array refcount underflow during %s", operation);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_cow_debug_abort(message);
    }
}

static PTN_UNUSED void ptn_cow_debug_assert_string_refcount(size_t *refcount, const char *operation) {
    if (refcount == NULL) {
        return;
    }
    if (*refcount == 0) {
        char message[128];
        int written = snprintf(message, sizeof(message), "string refcount underflow during %s", operation);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_cow_debug_abort(message);
    }
}

static PTN_UNUSED size_t ptn_cow_debug_live_count(size_t allocs, size_t frees, const char *name) {
    if (frees > allocs) {
        char message[128];
        int written = snprintf(message, sizeof(message), "%s frees exceed allocs", name);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_cow_debug_abort(message);
    }
    return allocs - frees;
}

static PTN_UNUSED void ptn_cow_debug_reset(void) {
    memset(&ptn_cow_debug_counters, 0, sizeof(ptn_cow_debug_counters));
}

static PTN_UNUSED int ptn_cow_debug_counter(const char *name, size_t *out) {
    if (strcmp(name, "string.alloc") == 0) {
        *out = ptn_cow_debug_counters.string_allocs;
        return 1;
    }
    if (strcmp(name, "string.free") == 0) {
        *out = ptn_cow_debug_counters.string_frees;
        return 1;
    }
    if (strcmp(name, "string.clone") == 0) {
        *out = ptn_cow_debug_counters.string_clones;
        return 1;
    }
    if (strcmp(name, "string.retain") == 0) {
        *out = ptn_cow_debug_counters.string_retain;
        return 1;
    }
    if (strcmp(name, "string.release") == 0) {
        *out = ptn_cow_debug_counters.string_release;
        return 1;
    }
    if (strcmp(name, "string.detach") == 0) {
        *out = ptn_cow_debug_counters.string_detaches;
        return 1;
    }
    if (strcmp(name, "string.live") == 0) {
        *out = ptn_cow_debug_live_count(
            ptn_cow_debug_counters.string_allocs,
            ptn_cow_debug_counters.string_frees,
            name
        );
        return 1;
    }
    if (strcmp(name, "array.alloc") == 0) {
        *out = ptn_cow_debug_counters.array_allocs;
        return 1;
    }
    if (strcmp(name, "array.retain") == 0) {
        *out = ptn_cow_debug_counters.array_retain;
        return 1;
    }
    if (strcmp(name, "array.release") == 0) {
        *out = ptn_cow_debug_counters.array_release;
        return 1;
    }
    if (strcmp(name, "array.free") == 0) {
        *out = ptn_cow_debug_counters.array_frees;
        return 1;
    }
    if (strcmp(name, "array.clone") == 0) {
        *out = ptn_cow_debug_counters.array_clones;
        return 1;
    }
    if (strcmp(name, "array.detach") == 0) {
        *out = ptn_cow_debug_counters.array_detaches;
        return 1;
    }
    if (strcmp(name, "array.detach_skip") == 0) {
        *out = ptn_cow_debug_counters.array_detach_skips;
        return 1;
    }
    if (strcmp(name, "array.live") == 0) {
        *out = ptn_cow_debug_live_count(
            ptn_cow_debug_counters.array_allocs,
            ptn_cow_debug_counters.array_frees,
            name
        );
        return 1;
    }
    return 0;
}

static PTN_UNUSED void ptn_cow_debug_assert_named_counter(const char *name, int64_t expected) {
    if (expected < 0) {
        ptn_cow_debug_abort_counter(name, 0, expected);
    }
    size_t actual = 0;
    if (!ptn_cow_debug_counter(name, &actual)) {
        ptn_cow_debug_abort("unknown counter");
    }
    if (actual != (size_t)expected) {
        ptn_cow_debug_abort_counter(name, actual, expected);
    }
}

static PTN_UNUSED void ptn_cow_debug_assert_balanced(void) {
    ptn_cow_debug_assert_named_counter("string.live", 0);
    ptn_cow_debug_assert_named_counter("array.live", 0);
}

static PTN_UNUSED char *ptn_duplicate_string(const char *string) {
    size_t len = strlen(string);
