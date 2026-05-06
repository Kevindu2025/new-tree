use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use std::fs::File;
use std::io::Read;
use digest::Digest;
use md5::Md5;
use sha1::Sha1;
use sha2::Sha256;
use zip::ZipArchive;
use tar::{Archive as TarArchive};
use flate2::read::GzDecoder;
use std::io::ErrorKind;
use sys_locale::get_locale;
use winreg::enums::*;
use winreg::{RegKey, HKEY};
use sevenz_rust;
use unrar::Archive as UnrarArchive;

// 语言枚举
#[derive(Debug, Clone, Copy, PartialEq)]
enum Language {
    Chinese,
    English,
    Russian,
}

// 语言资源
struct Resources {
    lang: Language,
}

impl Resources {
    fn new() -> Self {
        let lang = Self::detect_language();
        Resources { lang }
    }

    fn detect_language() -> Language {
        if let Some(locale) = get_locale() {
            if locale.starts_with("zh") {
                Language::Chinese
            } else if locale.starts_with("ru") {
                Language::Russian
            } else {
                Language::English
            }
        } else {
            Language::English
        }
    }

    // 错误消息
    fn error_empty_path(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "错误: 路径不能为空",
            Language::English => "Error: Path cannot be empty",
            Language::Russian => "Ошибка: Путь не может быть пустым",
        }
    }

    fn error_path_not_exist(&self, path: &str) -> String {
        match self.lang {
            Language::Chinese => format!("错误: 路径不存在: {}", path),
            Language::English => format!("Error: Path does not exist: {}", path),
            Language::Russian => format!("Ошибка: Путь не существует: {}", path),
        }
    }

    fn error_cannot_access(&self, path: &str, reason: &str) -> String {
        match self.lang {
            Language::Chinese => format!("错误: 无法访问路径: {} - {}", path, reason),
            Language::English => format!("Error: Cannot access path: {} - {}", path, reason),
            Language::Russian => format!("Ошибка: Не удается получить доступ к пути: {} - {}", path, reason),
        }
    }

    fn error_not_directory(&self, path: &str) -> String {
        match self.lang {
            Language::Chinese => format!("错误: 路径不是目录: {}", path),
            Language::English => format!("Error: Path is not a directory: {}", path),
            Language::Russian => format!("Ошибка: Путь не является каталогом: {}", path),
        }
    }

    fn error_cannot_read_dir(&self, path: &str, reason: &str) -> String {
        match self.lang {
            Language::Chinese => format!("错误: 无法读取目录: {} - {}", path, reason),
            Language::English => format!("Error: Cannot read directory: {} - {}", path, reason),
            Language::Russian => format!("Ошибка: Не удается прочитать каталог: {} - {}", path, reason),
        }
    }

    fn error_unknown_param(&self, param: &str) -> String {
        match self.lang {
            Language::Chinese => format!("错误: 未知参数: {}", param),
            Language::English => format!("Error: Unknown parameter: {}", param),
            Language::Russian => format!("Ошибка: Неизвестный параметр: {}", param),
        }
    }

    fn error_unsupported_hash(&self, hash_type: &str) -> String {
        match self.lang {
            Language::Chinese => format!("错误: 不支持的哈希类型: {}. 支持的类型: SIMPLE, CRC32, MD5, SHA1, SHA256", hash_type),
            Language::English => format!("Error: Unsupported hash type: {}. Supported types: SIMPLE, CRC32, MD5, SHA1, SHA256", hash_type),
            Language::Russian => format!("Ошибка: Неподдерживаемый тип хеша: {}. Поддерживаемые типы: SIMPLE, CRC32, MD5, SHA1, SHA256", hash_type),
        }
    }

    fn error_permission_denied(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "[无权访问]",
            Language::English => "[Access Denied]",
            Language::Russian => "[Доступ запрещен]",
        }
    }

    fn error_need_admin(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "[需要管理员权限]",
            Language::English => "[Administrator Privileges Required]",
            Language::Russian => "[Требуются права администратора]",
        }
    }

    fn error_cannot_get_file_info(&self, error: &str) -> String {
        match self.lang {
            Language::Chinese => format!("[无法获取文件信息: {}]", error),
            Language::English => format!("[Cannot get file info: {}]", error),
            Language::Russian => format!("[Не удается получить информацию о файле: {}]", error),
        }
    }

    fn error_read_error(&self, error: &str) -> String {
        match self.lang {
            Language::Chinese => format!("[读取错误: {}]", error),
            Language::English => format!("[Read error: {}]", error),
            Language::Russian => format!("[Ошибка чтения: {}]", error),
        }
    }

    fn error_no_match(&self, pattern: &str) -> String {
        match self.lang {
            Language::Chinese => format!("错误: 没有找到匹配 '{}' 的文件或目录", pattern),
            Language::English => format!("Error: No files or directories matching '{}' found", pattern),
            Language::Russian => format!("Ошибка: Файлы или каталоги, соответствующие '{}', не найдены", pattern),
        }
    }

    fn error_invalid_registry_path(&self, path: &str) -> String {
        match self.lang {
            Language::Chinese => format!("错误: 无效的注册表路径: {}", path),
            Language::English => format!("Error: Invalid registry path: {}", path),
            Language::Russian => format!("Ошибка: Неверный путь в реестре: {}", path),
        }
    }

    fn error_registry_access_denied(&self, path: &str) -> String {
        match self.lang {
            Language::Chinese => format!("错误: 无法访问注册表项: {} - 权限不足", path),
            Language::English => format!("Error: Cannot access registry key: {} - Access denied", path),
            Language::Russian => format!("Ошибка: Не удается получить доступ к разделу реестра: {} - Доступ запрещен", path),
        }
    }

    fn registry_path_label(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "注册表路径",
            Language::English => "Registry Path",
            Language::Russian => "Путь реестра",
        }
    }

    fn registry_key_label(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "[注册表项]",
            Language::English => "[Registry Key]",
            Language::Russian => "[Раздел реестра]",
        }
    }

    fn registry_value_label(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "[注册表值]",
            Language::English => "[Registry Value]",
            Language::Russian => "[Значение реестра]",
        }
    }

    fn help_param_r(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "   /R    列出注册表项和值（需要有效的注册表路径）。",
            Language::English => "   /R    List registry keys and values (requires valid registry path).",
            Language::Russian => "   /R    Перечислить ключи и значения реестра (требуется допустимый путь в реестре).",
        }
    }

    fn example_4(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "  TREE HKLM\\SOFTWARE /R        - 显示注册表项结构",
            Language::English => "  TREE HKLM\\SOFTWARE /R        - Display registry key structure",
            Language::Russian => "  TREE HKLM\\SOFTWARE /R        - Показать структуру ключей реестра",
        }
    }

    // 信息消息
    fn folder_path(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "文件夹路径",
            Language::English => "Folder Path",
            Language::Russian => "Путь к папке",
        }
    }

    fn archive_contents(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "(压缩包内容)",
            Language::English => "(Archive Contents)",
            Language::Russian => "(Содержимое архива)",
        }
    }

    fn single_compressed_file(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "单个压缩文件内容",
            Language::English => "Single Compressed File",
            Language::Russian => "Сжатый файл",
        }
    }

    fn help_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "以图形显示驱动器或路径的文件夹结构。",
            Language::English => "Graphically displays the folder structure of a drive or path.",
            Language::Russian => "Графически отображает структуру папок диска или пути.",
        }
    }

    fn help_usage(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "TREE [drive:][path] [/F] [/A] [/D] [/H] [/S] [/L] [/T:<type>] [/P:<pattern>] [/R] [/?]",
            Language::English => "TREE [drive:][path] [/F] [/A] [/D] [/H] [/S] [/L] [/T:<type>] [/P:<pattern>] [/R] [/?]",
            Language::Russian => "TREE [drive:][path] [/F] [/A] [/D] [/H] [/S] [/L] [/T:<type>] [/P:<pattern>] [/R] [/?]",
        }
    }

    fn help_param_f(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "   /F   显示每个文件夹中文件的名称。",
            Language::English => "   /F   Display the names of the files in each folder.",
            Language::Russian => "   /F   Отображать имена файлов в каждой папке.",
        }
    }

    fn help_param_a(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "   /A   使用 ASCII 字符，而不使用扩展字符。",
            Language::English => "   /A   Use ASCII characters instead of extended characters.",
            Language::Russian => "   /A   Использовать символы ASCII вместо расширенных.",
        }
    }

    fn help_param_d(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "   /D   显示文件详细信息（大小、时间）。",
            Language::English => "   /D   Display file details (size, time).",
            Language::Russian => "   /D   Отображать сведения о файлах (размер, время).",
        }
    }

    fn help_param_h(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "   /H   计算文件哈希值。",
            Language::English => "   /H   Calculate file hash values.",
            Language::Russian => "   /H   Вычислить хеши файлов.",
        }
    }

    fn help_param_s(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "   /S   显示隐藏文件。",
            Language::English => "   /S   Show hidden files.",
            Language::Russian => "   /S   Показать скрытые файлы.",
        }
    }

    fn help_param_l(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "   /L   列出压缩包内部结构。",
            Language::English => "   /L   List archive internal structure.",
            Language::Russian => "   /L   Показать внутреннюю структуру архивов.",
        }
    }

    fn help_param_t(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "   /T:  指定哈希类型 (SIMPLE, CRC32, MD5, SHA1, SHA256)。",
            Language::English => "   /T:  Specify hash type (SIMPLE, CRC32, MD5, SHA1, SHA256).",
            Language::Russian => "   /T:  Указать тип хеша (SIMPLE, CRC32, MD5, SHA1, SHA256).",
        }
    }

    fn help_param_p(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "   /P:  模糊搜索文件名或目录名。",
            Language::English => "   /P:  Fuzzy search for file or directory names.",
            Language::Russian => "   /P:  Нечеткий поиск по именам файлов или каталогов.",
        }
    }

    fn help_param_q(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "   /?   显示此帮助信息。",
            Language::English => "   /?   Display this help message.",
            Language::Russian => "   /?   Показать это справочное сообщение.",
        }
    }

    fn help_supported_formats(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "支持的压缩包格式",
            Language::English => "Supported Archive Formats",
            Language::Russian => "Поддерживаемые форматы архивов",
        }
    }

    fn help_permission_info(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "权限说明",
            Language::English => "Permission Information",
            Language::Russian => "Информация о разрешениях",
        }
    }

    fn help_examples(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "示例",
            Language::English => "Examples",
            Language::Russian => "Примеры",
        }
    }

    fn example_1(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "  TREE C:\\ /F /H /T:SHA256    - 显示C盘所有文件及其SHA256哈希",
            Language::English => "  TREE C:\\ /F /H /T:SHA256    - Display all files on C: drive with SHA256 hash",
            Language::Russian => "  TREE C:\\ /F /H /T:SHA256    - Показать все файлы на диске C: с хешем SHA256",
        }
    }

    fn example_2(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "  TREE . /D /H /T:MD5         - 显示当前目录详细信息及MD5哈希",
            Language::English => "  TREE . /D /H /T:MD5         - Display current directory details with MD5 hash",
            Language::Russian => "  TREE . /D /H /T:MD5         - Показать текущий каталог с деталями и хешем MD5",
        }
    }

    fn example_3(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "  TREE . /P:src               - 模糊搜索包含'src'的目录/文件",
            Language::English => "  TREE . /P:src               - Fuzzy search for dirs/files containing 'src'",
            Language::Russian => "  TREE . /P:src               - Нечеткий поиск каталогов/файлов с 'src'",
        }
    }

    fn hash_label_simple(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "校验和",
            Language::English => "Checksum",
            Language::Russian => "Контрольная сумма",
        }
    }

    fn hash_label_crc(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "CRC32",
            Language::English => "CRC32",
            Language::Russian => "CRC32",
        }
    }

    fn hash_label_md5(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "MD5",
            Language::English => "MD5",
            Language::Russian => "MD5",
        }
    }

    fn hash_label_sha1(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "SHA1",
            Language::English => "SHA1",
            Language::Russian => "SHA1",
        }
    }

    fn hash_label_sha256(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "SHA256",
            Language::English => "SHA256",
            Language::Russian => "SHA256",
        }
    }

    fn bytes_label(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "字节",
            Language::English => "bytes",
            Language::Russian => "байт",
        }
    }

    fn search_result_label(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "搜索结果",
            Language::English => "Search Results",
            Language::Russian => "Результаты поиска",
        }
    }
}

// 定义支持的哈希类型
#[derive(Debug, PartialEq)]
enum HashType {
    Simple,
    Crc32,
    Md5,
    Sha1,
    Sha256,
}

// 命令行参数结构
struct Args {
    show_files: bool,
    use_ascii: bool,
    show_details: bool,
    compute_hash: bool,
    hash_type: HashType,
    target_path: PathBuf,
    show_help: bool,
    show_hidden: bool,
    list_archive: bool,
    is_admin: bool,
    resources: Resources,
    search_pattern: Option<String>,
    show_registry: bool,
}

impl Args {
    fn new() -> Self {
        let is_admin = cfg!(windows) && std::env::var_os("USERNAME").map_or(false, |u| u == "Administrator");
        let resources = Resources::new();
        
        Args {
            show_files: false,
            use_ascii: false,
            show_details: false,
            compute_hash: false,
            hash_type: HashType::Simple,
            target_path: PathBuf::from("."),
            show_help: false,
            show_hidden: false,
            list_archive: false,
            is_admin,
            resources,
            search_pattern: None,
            show_registry: false,
        }
    }

    // 解析命令行参数
    fn parse(&mut self, args: &[String]) -> Result<(), String> {
        let mut i = 1;
        while i < args.len() {
            let arg = &args[i];
            match arg.to_uppercase().as_str() {
                "/F" => self.show_files = true,
                "/A" => self.use_ascii = true,
                "/D" => self.show_details = true,
                "/H" => self.compute_hash = true,
                "/S" => self.show_hidden = true,
                "/L" => self.list_archive = true,
                "/?" => self.show_help = true,
                "/R" => self.show_registry = true,
                _ => {
                    if arg.starts_with("/T:") {
                        let hash_type = arg[3..].to_uppercase();
                        self.hash_type = match hash_type.as_str() {
                            "SIMPLE" => HashType::Simple,
                            "CRC32" => HashType::Crc32,
                            "MD5" => HashType::Md5,
                            "SHA1" => HashType::Sha1,
                            "SHA256" => HashType::Sha256,
                            _ => return Err(self.resources.error_unsupported_hash(&hash_type)),
                        };
                    } else if arg.starts_with("/P:") {
                        let pattern = arg[3..].trim().to_string();
                        if !pattern.is_empty() {
                            self.search_pattern = Some(pattern);
                        }
                    } else if arg.starts_with("/") {
                        return Err(self.resources.error_unknown_param(arg));
                    } else {
                        self.target_path = PathBuf::from(arg);
                    }
                }
            }
            i += 1;
        }
        Ok(())
    }
}

// 计算简单校验和
fn calculate_simple_hash(data: &[u8]) -> String {
    let sum: u32 = data.iter().map(|&b| b as u32).sum();
    format!("chk{:08x}", sum)
}

// 计算CRC32
fn calculate_crc32(data: &[u8]) -> String {
    let mut crc: u32 = 0xFFFFFFFF;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc = !crc;
    format!("crc{:08x}", crc)
}

// 计算MD5
fn calculate_md5(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

// 计算SHA1
fn calculate_sha1(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

// 计算SHA256
fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

// 获取格式化的时间戳
fn format_timestamp(timestamp: u64) -> String {
    let datetime = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
    match datetime.checked_add(std::time::Duration::from_secs(8 * 3600)) {
        Some(dt) => {
            let time = OffsetDateTime::from(dt);
            format!("{}", time.format(&Rfc3339).unwrap_or_else(|_| "Unknown Time".to_string()))
        }
        None => "Unknown Time".to_string(),
    }
}

// 检查文件/目录访问权限
fn check_access(path: &Path, is_admin: bool, resources: &Resources) -> Result<(), String> {
    match fs::metadata(path) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.kind() == ErrorKind::PermissionDenied {
                if is_admin {
                    Err(resources.error_need_admin().to_string())
                } else {
                    Err(resources.error_permission_denied().to_string())
                }
            } else {
                Err(format!("Cannot access: {}", e))
            }
        }
    }
}

// 打印树形结构
fn print_tree(
    dir: &Path,
    prefix: &str,
    is_last: bool,
    is_root: bool,
    args: &Args,
) {
    // 检查访问权限
    if let Err(err) = check_access(dir, args.is_admin, &args.resources) {
        let name = if is_root {
            dir.display().to_string()
        } else {
            dir.file_name().unwrap_or_default().to_string_lossy().to_string()
        };
        println!("{}{}{} {}", prefix, 
            if is_last { get_end_symbol(args.use_ascii) } else { get_mid_symbol(args.use_ascii) },
            name,
            err);
        return;
    }

    if !dir.is_dir() {
        return;
    }

    // 如果有搜索模式，检查当前目录是否匹配
    let search_pattern = args.search_pattern.as_deref();
    if let Some(pattern) = search_pattern {
        if !is_root && !should_include_path(dir, pattern) {
            return;
        }
    }

    let entries_result = fs::read_dir(dir);
    let entries: Vec<_> = match entries_result {
        Ok(iter) => iter.filter_map(|entry| entry.ok()).collect(),
        Err(e) => {
            if e.kind() == ErrorKind::PermissionDenied {
                let name = if is_root {
                    dir.display().to_string()
                } else {
                    dir.file_name().unwrap_or_default().to_string_lossy().to_string()
                };
                if args.is_admin {
                    println!("{}{}{} {}", prefix, 
                        if is_last { get_end_symbol(args.use_ascii) } else { get_mid_symbol(args.use_ascii) },
                        name,
                        args.resources.error_need_admin());
                } else {
                    println!("{}{}{} {}", prefix, 
                        if is_last { get_end_symbol(args.use_ascii) } else { get_mid_symbol(args.use_ascii) },
                        name,
                        args.resources.error_permission_denied());
                }
                return;
            }
            vec![]
        }
    };

    let filtered_entries: Vec<_> = entries.into_iter()
        .filter(|entry| {
            if args.show_hidden {
                true
            } else {
                let name = entry.file_name().to_string_lossy().to_string();
                !name.starts_with('.') && name != "desktop.ini"
            }
        })
        .map(|entry| entry.path())
        .collect();

    let (dirs, files): (Vec<_>, Vec<_>) = filtered_entries.into_iter().partition(|p| p.is_dir());

    // 打印当前目录
    let connector = if is_last { get_end_symbol(args.use_ascii) } else { get_mid_symbol(args.use_ascii) };
    let name = if is_root {
        dir.display().to_string()
    } else {
        dir.file_name().unwrap_or_default().to_string_lossy().to_string()
    };
    println!("{}{}{}", prefix, connector, name);

    let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { get_branch_symbol(args.use_ascii) });

    // 过滤并打印子目录
    let filtered_dirs: Vec<PathBuf> = if let Some(pattern) = search_pattern {
        dirs.into_iter()
            .filter(|d| has_matching_content(d, pattern, args.show_files))
            .collect()
    } else {
        dirs
    };

    for (i, child_dir) in filtered_dirs.iter().enumerate() {
        let is_last_child = i == filtered_dirs.len() - 1 && files.is_empty();
        print_tree(child_dir, &new_prefix, is_last_child, false, args);
    }

    // 过滤并打印文件（如果请求）
    if args.show_files {
        let filtered_files: Vec<PathBuf> = if let Some(pattern) = search_pattern {
            files.into_iter()
                .filter(|f| should_include_path(f, pattern))
                .collect()
        } else {
            files
        };

        for (i, file) in filtered_files.iter().enumerate() {
            let is_last_file = i == filtered_files.len() - 1;
            let file_connector = if is_last_file { get_end_symbol(args.use_ascii) } else { get_mid_symbol(args.use_ascii) };

            // 检查文件访问权限
            if let Err(err) = check_access(file, args.is_admin, &args.resources) {
                println!("{}{}{} {}", &new_prefix, file_connector, file.file_name().unwrap_or_default().to_string_lossy(), err);
                continue;
            }

            if args.list_archive && is_archive_file(file) {
                print_archive_contents(file, &new_prefix, file_connector, args);
            } else if args.compute_hash {
                print_file_with_hash(file, &new_prefix, file_connector, args);
            } else if args.show_details {
                print_file_details(file, &new_prefix, file_connector, args);
            } else {
                println!("{}{}{}", &new_prefix, file_connector, file.file_name().unwrap_or_default().to_string_lossy());
            }
        }
    }
}

// 检查目录是否包含匹配的内容
fn has_matching_content(dir: &Path, pattern: &str, include_files: bool) -> bool {
    // 首先检查目录名是否匹配
    if should_include_path(dir, pattern) {
        return true;
    }

    // 检查子目录和文件
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if has_matching_content(&path, pattern, include_files) {
                    return true;
                }
            } else if include_files && path.is_file() {
                if should_include_path(&path, pattern) {
                    return true;
                }
            }
        }
    }
    false
}

// 检查是否为支持的压缩包文件
fn is_archive_file(file: &Path) -> bool {
    let ext = file.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    matches!(ext.as_ref(), "zip" | "tar" | "gz" | "tgz" | "7z" | "rar" | "wim")
}

// 打印压缩包内部结构
fn print_archive_contents(file: &Path, prefix: &str, connector: &str, args: &Args) {
    let file_name = file.file_name().unwrap_or_default().to_string_lossy();
    println!("{}{}{} {}", prefix, connector, file_name, args.resources.archive_contents());

    let new_prefix = format!("{}{}", prefix, get_branch_symbol(args.use_ascii));

    let ext = file.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    match ext.as_ref() {
        "zip" => {
            if let Ok(file) = File::open(file) {
                if let Ok(mut archive) = ZipArchive::new(file) {
                    let total_entries = archive.len();
                    for i in 0..total_entries {
                        if let Ok(entry) = archive.by_index(i) {
                            let name = entry.name().to_string();
                            let is_last = i == total_entries - 1;
                            let entry_connector = if is_last { get_end_symbol(args.use_ascii) } else { get_mid_symbol(args.use_ascii) };
                            println!("{}{}{}", new_prefix, entry_connector, name);
                        }
                    }
                }
            }
        }
        "tar" | "tgz" => {
            if let Ok(file) = File::open(file) {
                let entries: Vec<String> = if ext == "tgz" {
                    // 处理 .tgz 文件
                    let gz = GzDecoder::new(file);
                    let mut archive = TarArchive::new(gz);
                    archive.entries()
                        .map(|entries| {
                            entries.filter_map(|entry| {
                                entry.ok().and_then(|e| e.path().ok().map(|p| p.to_string_lossy().to_string()))
                            }).collect()
                        })
                        .unwrap_or_default()
                } else {
                    // 处理 .tar 文件
                    let mut archive = TarArchive::new(file);
                    archive.entries()
                        .map(|entries| {
                            entries.filter_map(|entry| {
                                entry.ok().and_then(|e| e.path().ok().map(|p| p.to_string_lossy().to_string()))
                            }).collect()
                        })
                        .unwrap_or_default()
                };
                
                let total_entries = entries.len();
                for (i, name) in entries.iter().enumerate() {
                    let is_last = i == total_entries - 1;
                    let entry_connector = if is_last { get_end_symbol(args.use_ascii) } else { get_mid_symbol(args.use_ascii) };
                    println!("{}{}{}", new_prefix, entry_connector, name);
                }
            }
        }
        "gz" => {
            // 处理 .gz 文件
            if let Ok(file) = File::open(file) {
                let gz = GzDecoder::new(file);
                let mut buffer = Vec::new();
                if gz.take(1024).read_to_end(&mut buffer).is_ok() {
                    println!("{}{}{}", new_prefix, get_end_symbol(args.use_ascii), args.resources.single_compressed_file());
                }
            }
        }
        "7z" => {
            // 处理 7z 文件 - 解压到临时目录再列出
            let temp_dir = std::env::temp_dir().join(format!("rs_7z_{}", std::process::id()));
            if let Ok(_) = sevenz_rust::decompress_file(file, &temp_dir) {
                if let Ok(entries) = fs::read_dir(&temp_dir) {
                    let all_entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                    let total_entries = all_entries.len();
                    for (i, entry) in all_entries.iter().enumerate() {
                        let is_last = i == total_entries - 1;
                        let entry_connector = if is_last { get_end_symbol(args.use_ascii) } else { get_mid_symbol(args.use_ascii) };
                        let name = entry.file_name().to_string_lossy().to_string();
                        if entry.path().is_dir() {
                            println!("{}{}{} [7z Folder]", new_prefix, entry_connector, name);
                        } else {
                            println!("{}{}{}", new_prefix, entry_connector, name);
                        }
                    }
                }
                // 清理临时目录
                let _ = fs::remove_dir_all(&temp_dir);
            }
        }
        "rar" => {
            // 处理 RAR 文件
            let archive = UnrarArchive::new(file);
            if let Ok(entries) = archive.open_for_listing() {
                let all_entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                let total_entries = all_entries.len();
                for (i, entry) in all_entries.iter().enumerate() {
                    let is_last = i == total_entries - 1;
                    let entry_connector = if is_last { get_end_symbol(args.use_ascii) } else { get_mid_symbol(args.use_ascii) };
                    let name = entry.filename.to_string_lossy().to_string();
                    println!("{}{}{}", new_prefix, entry_connector, name);
                }
            }
        }
        "wim" => {
            // 处理 WIM 文件 - 使用 PowerShell
            println!("{}{}WIM archive detected", new_prefix, get_mid_symbol(args.use_ascii));
        }
        _ => {}
    }
}

// 打印文件详细信息
fn print_file_details(file: &Path, prefix: &str, connector: &str, args: &Args) {
    match fs::metadata(file) {
        Ok(metadata) => {
            let size = metadata.len();
            let modified = metadata.modified()
                .unwrap_or_else(|_| SystemTime::now())
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let datetime = format_timestamp(modified);

            println!("{}{}{} ({} {}, {})", prefix, connector, file.file_name().unwrap_or_default().to_string_lossy(), size, args.resources.bytes_label(), datetime);
        }
        Err(e) => {
            if e.kind() == ErrorKind::PermissionDenied {
                if args.is_admin {
                    println!("{}{}{} {}", prefix, connector, file.file_name().unwrap_or_default().to_string_lossy(), args.resources.error_need_admin());
                } else {
                    println!("{}{}{} {}", prefix, connector, file.file_name().unwrap_or_default().to_string_lossy(), args.resources.error_permission_denied());
                }
            } else {
                println!("{}{}{} {}", prefix, connector, file.file_name().unwrap_or_default().to_string_lossy(), args.resources.error_cannot_get_file_info(&e.to_string()));
            }
        }
    }
}

// 打印带哈希的文件信息
fn print_file_with_hash(file: &Path, prefix: &str, connector: &str, args: &Args) {
    match fs::metadata(file) {
        Ok(metadata) => {
            let size = metadata.len();
            let modified = metadata.modified()
                .unwrap_or_else(|_| SystemTime::now())
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let datetime = format_timestamp(modified);

            let hash_value = if args.compute_hash {
                match fs::read(file) {
                    Ok(data) => {
                        match args.hash_type {
                            HashType::Simple => calculate_simple_hash(&data),
                            HashType::Crc32 => calculate_crc32(&data),
                            HashType::Md5 => calculate_md5(&data),
                            HashType::Sha1 => calculate_sha1(&data),
                            HashType::Sha256 => calculate_sha256(&data),
                        }
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::PermissionDenied {
                            if args.is_admin {
                                args.resources.error_need_admin().to_string()
                            } else {
                                args.resources.error_permission_denied().to_string()
                            }
                        } else {
                            args.resources.error_read_error(&e.to_string())
                        }
                    }
                }
            } else {
                String::new()
            };

            let hash_label = match args.hash_type {
                HashType::Simple => args.resources.hash_label_simple(),
                HashType::Crc32 => args.resources.hash_label_crc(),
                HashType::Md5 => args.resources.hash_label_md5(),
                HashType::Sha1 => args.resources.hash_label_sha1(),
                HashType::Sha256 => args.resources.hash_label_sha256(),
            };

            if args.show_details {
                println!("{}{}{} ({} {}, {}) [{}: {}]", prefix, connector, file.file_name().unwrap_or_default().to_string_lossy(), size, args.resources.bytes_label(), datetime, hash_label, hash_value);
            } else {
                println!("{}{}{} [{}: {}]", prefix, connector, file.file_name().unwrap_or_default().to_string_lossy(), hash_label, hash_value);
            }
        }
        Err(e) => {
            if e.kind() == ErrorKind::PermissionDenied {
                if args.is_admin {
                    println!("{}{}{} {}", prefix, connector, file.file_name().unwrap_or_default().to_string_lossy(), args.resources.error_need_admin());
                } else {
                    println!("{}{}{} {}", prefix, connector, file.file_name().unwrap_or_default().to_string_lossy(), args.resources.error_permission_denied());
                }
            } else {
                println!("{}{}{} {}", prefix, connector, file.file_name().unwrap_or_default().to_string_lossy(), args.resources.error_cannot_get_file_info(&e.to_string()));
            }
        }
    }
}

// 获取符号
fn get_end_symbol(use_ascii: bool) -> &'static str {
    if use_ascii {
        "+-- "
    } else {
        "└── "
    }
}

fn get_mid_symbol(use_ascii: bool) -> &'static str {
    if use_ascii {
        "+-- "
    } else {
        "├── "
    }
}

fn get_branch_symbol(use_ascii: bool) -> &'static str {
    if use_ascii {
        "|   "
    } else {
        "│   "
    }
}

// 模糊匹配函数
fn fuzzy_match(name: &str, pattern: &str) -> bool {
    let name_lower = name.to_lowercase();
    let pattern_lower = pattern.to_lowercase();
    name_lower.contains(&pattern_lower)
}

// 检查路径是否匹配搜索模式或其子目录包含匹配项
fn should_include_path(path: &Path, pattern: &str) -> bool {
    let name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());
    fuzzy_match(&name, pattern)
}

// 检查路径是否为注册表路径
fn is_registry_path(path: &str) -> bool {
    let upper = path.to_uppercase();
    upper.starts_with("HKLM\\") || upper.starts_with("HKCU\\") ||
    upper.starts_with("HKEY_LOCAL_MACHINE\\") || upper.starts_with("HKEY_CURRENT_USER\\") ||
    upper.starts_with("HKCR\\") || upper.starts_with("HKEY_CLASSES_ROOT\\") ||
    upper.starts_with("HKU\\") || upper.starts_with("HKEY_USERS\\") ||
    upper.starts_with("HKCC\\") || upper.starts_with("HKEY_CURRENT_CONFIG\\")
}

// 解析注册表路径，返回 (hkey, subkey)
fn parse_registry_path(path: &str) -> Result<(HKEY, String), String> {
    let upper = path.to_uppercase();
    
    let (hkey, subkey) = if upper.starts_with("HKLM\\") || upper.starts_with("HKEY_LOCAL_MACHINE\\") {
        let key = if upper.starts_with("HKLM\\") {
            (HKEY_LOCAL_MACHINE, &path[5..])
        } else {
            (HKEY_LOCAL_MACHINE, &path[21..])
        };
        key
    } else if upper.starts_with("HKCU\\") || upper.starts_with("HKEY_CURRENT_USER\\") {
        let key = if upper.starts_with("HKCU\\") {
            (HKEY_CURRENT_USER, &path[5..])
        } else {
            (HKEY_CURRENT_USER, &path[21..])
        };
        key
    } else if upper.starts_with("HKCR\\") || upper.starts_with("HKEY_CLASSES_ROOT\\") {
        let key = if upper.starts_with("HKCR\\") {
            (HKEY_CLASSES_ROOT, &path[5..])
        } else {
            (HKEY_CLASSES_ROOT, &path[21..])
        };
        key
    } else if upper.starts_with("HKU\\") || upper.starts_with("HKEY_USERS\\") {
        let key = if upper.starts_with("HKU\\") {
            (HKEY_USERS, &path[4..])
        } else {
            (HKEY_USERS, &path[13..])
        };
        key
    } else if upper.starts_with("HKCC\\") || upper.starts_with("HKEY_CURRENT_CONFIG\\") {
        let key = if upper.starts_with("HKCC\\") {
            (HKEY_CURRENT_CONFIG, &path[5..])
        } else {
            (HKEY_CURRENT_CONFIG, &path[22..])
        };
        key
    } else {
        return Err("Invalid registry path".to_string());
    };
    
    Ok((hkey, subkey.to_string()))
}

// 打印注册表树
fn print_registry_tree(
    hkey: HKEY,
    subkey: &str,
    prefix: &str,
    is_last: bool,
    is_root: bool,
    args: &Args,
    resources: &Resources,
) {
    let key_name = if is_root {
        subkey.to_string()
    } else {
        subkey.split('\\').last().unwrap_or(subkey).to_string()
    };

    let connector = if is_last { get_end_symbol(args.use_ascii) } else { get_mid_symbol(args.use_ascii) };
    println!("{}{}{} {}", prefix, connector, key_name, resources.registry_key_label());

    let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { get_branch_symbol(args.use_ascii) });

    // 尝试打开注册表项
    let reg_key = match RegKey::predef(hkey).open_subkey(subkey) {
        Ok(key) => key,
        Err(e) => {
            if e.kind() == ErrorKind::PermissionDenied {
                println!("{}{}{} {}", new_prefix, get_end_symbol(args.use_ascii), subkey.split('\\').last().unwrap_or(""), resources.error_permission_denied());
            }
            return;
        }
    };

    // 打印子项
    let subkeys: Vec<String> = reg_key.enum_keys()
        .filter_map(|k| k.ok())
        .collect();

    // 打印值
    let values: Vec<_> = reg_key.enum_values()
        .filter_map(|v| v.ok())
        .collect();
    let total_items = subkeys.len() + values.len();

    if total_items == 0 {
        return;
    }

    // 打印子项
    for (i, sub_key_name) in subkeys.iter().enumerate() {
        let is_last_item = i == subkeys.len() - 1 && values.is_empty();
        let full_subkey = if subkey.is_empty() {
            sub_key_name.clone()
        } else {
            format!("{}\\{}", subkey, sub_key_name)
        };
        print_registry_tree(hkey, &full_subkey, &new_prefix, is_last_item, false, args, resources);
    }

    // 打印值
    for (i, (value_name, _)) in values.iter().enumerate() {
        let is_last_value = i == values.len() - 1;
        let value_connector = if is_last_value { get_end_symbol(args.use_ascii) } else { get_mid_symbol(args.use_ascii) };
        let display_name = if value_name.is_empty() {
            "(默认)".to_string()
        } else {
            value_name.clone()
        };
        println!("{}{}{} {}", new_prefix, value_connector, display_name, resources.registry_value_label());
    }
}

// 处理注册表路径
fn handle_registry(path: &str, args: &Args, resources: &Resources) -> Result<(), String> {
    if !is_registry_path(path) {
        return Err(resources.error_invalid_registry_path(path));
    }

    let (hkey, subkey) = parse_registry_path(path)?;
    
    println!("{}: {}", resources.registry_path_label(), path);
    println!();

    // 对于注册表，忽略不兼容的参数
    let mut args_copy = args.clone();
    args_copy.show_files = false;
    args_copy.show_details = false;
    args_copy.compute_hash = false;
    args_copy.list_archive = false;

    print_registry_tree(hkey, &subkey, "", true, true, &args_copy, resources);
    Ok(())
}

// 克隆 Args 以便在函数中传递
impl Clone for Args {
    fn clone(&self) -> Self {
        Args {
            show_files: self.show_files,
            use_ascii: self.use_ascii,
            show_details: self.show_details,
            compute_hash: self.compute_hash,
            hash_type: self.hash_type.clone(),
            target_path: self.target_path.clone(),
            show_help: self.show_help,
            show_hidden: self.show_hidden,
            list_archive: self.list_archive,
            is_admin: self.is_admin,
            resources: self.resources.clone(),
            search_pattern: self.search_pattern.clone(),
            show_registry: self.show_registry,
        }
    }
}

// 克隆 HashType
impl Clone for HashType {
    fn clone(&self) -> Self {
        match self {
            HashType::Simple => HashType::Simple,
            HashType::Crc32 => HashType::Crc32,
            HashType::Md5 => HashType::Md5,
            HashType::Sha1 => HashType::Sha1,
            HashType::Sha256 => HashType::Sha256,
        }
    }
}

// 克隆 Resources
impl Clone for Resources {
    fn clone(&self) -> Self {
        Resources {
            lang: self.lang,
        }
    }
}

// 显示帮助信息
fn show_help(resources: &Resources) {
    println!("{}", resources.help_title());
    println!();
    println!("{}", resources.help_usage());
    println!();
    println!("{}", resources.help_param_f());
    println!("{}", resources.help_param_a());
    println!("{}", resources.help_param_d());
    println!("{}", resources.help_param_h());
    println!("{}", resources.help_param_s());
    println!("{}", resources.help_param_l());
    println!("{}", resources.help_param_t());
    println!("{}", resources.help_param_p());
    println!("{}", resources.help_param_r());
    println!("{}", resources.help_param_q());
    println!();
    println!("{}: ZIP, TAR, GZ, TGZ, 7Z, RAR, WIM", resources.help_supported_formats());
    println!();
    println!("{}", resources.help_permission_info());
    println!();
    println!("{}", resources.help_examples());
    println!("{}", resources.example_1());
    println!("{}", resources.example_2());
    println!("{}", resources.example_3());
    println!("{}", resources.example_4());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cli_args = Args::new();

    // 解析命令行参数
    if let Err(err) = cli_args.parse(&args) {
        eprintln!("{}", err);
        std::process::exit(1);
    }

    // 如果请求帮助，显示帮助并退出
    if cli_args.show_help {
        show_help(&cli_args.resources);
        return;
    }

    let path = &cli_args.target_path;
    let path_str = path.to_string_lossy();

    // 检查是否为注册表路径或使用了 /R 参数
    let is_registry = cli_args.show_registry || is_registry_path(&path_str);

    if is_registry {
        // 处理注册表
        if !is_registry_path(&path_str) {
            eprintln!("{}", cli_args.resources.error_invalid_registry_path(&path_str));
            std::process::exit(1);
        }
        
        if let Err(err) = handle_registry(&path_str, &cli_args, &cli_args.resources) {
            eprintln!("{}", err);
            std::process::exit(1);
        }
        return;
    }

    // ========== 目录路径验证逻辑 ==========
    
    // 1. 检查路径是否为空
    if path_str.trim().is_empty() {
        eprintln!("{}", cli_args.resources.error_empty_path());
        std::process::exit(1);
    }

    // 2. 检查路径是否存在
    if !path.exists() {
        eprintln!("{}", cli_args.resources.error_path_not_exist(&path_str));
        std::process::exit(1);
    }

    // 3. 检查路径是否可访问
    if let Err(err) = check_access(path, cli_args.is_admin, &cli_args.resources) {
        eprintln!("{}", cli_args.resources.error_cannot_access(&path_str, &err));
        std::process::exit(1);
    }

    // 4. 检查路径是否为目录
    if !path.is_dir() {
        eprintln!("{}", cli_args.resources.error_not_directory(&path_str));
        std::process::exit(1);
    }

    // 5. 检查路径是否可读
    match fs::read_dir(path) {
        Ok(_) => {},
        Err(e) => {
            if e.kind() == ErrorKind::PermissionDenied {
                if cli_args.is_admin {
                    eprintln!("{}", cli_args.resources.error_cannot_read_dir(&path_str, cli_args.resources.error_need_admin()));
                } else {
                    eprintln!("{}", cli_args.resources.error_cannot_read_dir(&path_str, cli_args.resources.error_permission_denied()));
                }
            } else {
                eprintln!("{}", cli_args.resources.error_cannot_read_dir(&path_str, &e.to_string()));
            }
            std::process::exit(1);
        }
    }

    // ========== 路径验证结束 ==========

    let canonical_path = path.canonicalize().unwrap();
    println!("{}: {}", cli_args.resources.folder_path(), canonical_path.display());
    println!();

    print_tree(&canonical_path, "", true, true, &cli_args);
}