RS - Directory Tree Viewer

A Rust program similar to the Windows TREE command, designed to graphically display directory structures, with support for file hash calculation and archive content viewing.

✨ Features

Graphically displays directory tree structure
Supports displaying files
Shows detailed file information (size, time)
Calculates file hash values
Displays hidden files
Lists internal archive structure (ZIP, TAR, GZ, TGZ)
Registry Viewer - Displays registry keys and values in a directory tree format
Multi-language Support (Chinese/English/Russian, automatically selects based on system locale)
Fuzzy Search - Displays only matching files or directories

🔐 Supported Hash Algorithms

SIMPLE - Simple checksum
CRC32 - CRC32 checksum
MD5 - MD5 hash
SHA1 - SHA1 hash
SHA256 - SHA256 hash

🛠 Usage

TREE [drive:][path] [/F] [/A] [/D] [/H] [/S] [/L] [/T:] [/P:] [/?]

📝 Parameter Description
Parameter   Description
path   The directory path or registry path to display

/F   Display the name of files in each folder

/A   Use ASCII characters instead of extended characters

/D   Display detailed file information (size, time)

/H   Calculate file hash values

/S   Display hidden files

/L   List internal archive structure

/T:   Specify hash type (SIMPLE, CRC32, MD5, SHA1, SHA256)

/P:   Fuzzy search for file or directory names (case-insensitive)

/R   List registry keys and values (requires a valid registry path)

/?   Display help information

🚀 Examples

Display current directory tree structure
RS .

Display all files
RS C:pathtodir /F

Display detailed information (size, time)
RS . /D

Calculate SHA256 hash for all files
RS . /F /H /T:SHA256

Display hidden files and calculate MD5
RS . /F /S /H /T:MD5

List archive contents
RS . /L

Fuzzy search - display directories and files containing "src"
RS . /F /P:src

Fuzzy search - display directory tree containing "test"
RS . /P:test

View registry (using /R parameter)
RS HKLMSOFTWARE /R

View registry (auto-detect registry path)
RS HKCUEnvironment /R

Display help
RS /?

🌐 Multi-language Support

The program automatically selects the interface language based on the system locale:

Chinese (zh*) - Chinese interface
English (en*) - English interface
Russian (ru*) - Russian interface

🏗 Building

Requires the Rust toolchain to be installed:

cargo build --release

📦 Dependencies

time - Time processing
zip - ZIP archive support
tar - TAR archive support
flate2 - GZIP compression support
digest - Hash calculation abstraction layer
md-5 - MD5 hash
sha1 - SHA1 hash
sha2 - SHA256 hash
sys-locale - System locale detection
