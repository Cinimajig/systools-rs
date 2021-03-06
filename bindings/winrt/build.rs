fn main() {
    windows::build!(
        windows::ui::notifications::*,
        windows::win32::windows_programming::{RegOpenKeyExW, RegSetValueExA, RegSetValueExW, RegCloseKey, HKEY, RegCreateKeyExW}
    );
}
