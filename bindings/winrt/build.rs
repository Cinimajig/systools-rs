fn main() {
    windows::build!(
        windows::ui::notifications::*,
        windows::win32::windows_programming::{RegOpenKeyExW, RegSetValueExA, RegSetValueExW, RegCloseKey, HKEY, RegCreateKeyExW},
        windows::win32::shell::{IShellLinkW, ShellLink},
        windows::win32::com::{CoCreateInstance, CoUninitialize, IPersistFile},
        windows::win32::file_system::GetFullPathNameW
    );
}
