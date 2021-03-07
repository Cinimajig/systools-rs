#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::Interface;
use windows::{
    data::xml::dom::XmlDocument,
    ui::notifications::{ToastNotification, ToastNotificationManager, ToastTemplateType},
};
use winrt::windows;
use wstring::WideString;

use std::env;

const XML_TEMPLATE: &str = r#"
<toast>
  <visual>
    <binding template="ToastImageAndText02">
      <image id="1" src="assets\lame_sun.png" />
      <text id="1"></text>
      <text id="2"></text>
    </binding>
  </visual>
  <audio src="ms-winsoundevent:Notification.Default" />
</toast>
"#;

/// Helper struct, to help with CoInitialize and CoUnitialize of COM.
struct InitCom();

impl InitCom {
    #![allow(dead_code)]
    /// Single-Threaded Application.
    fn sta() -> windows::Result<Self> {
        if let Err(err) = windows::initialize_sta() {
            Err(err)
        } else {
            Ok(Self())
        }
    }

    /// Multi-Threaded Application.
    fn mta() -> windows::Result<Self> {
        if let Err(err) = windows::initialize_mta() {
            Err(err)
        } else {
            Ok(Self())
        }
    }
}

impl Drop for InitCom {
    fn drop(&mut self) {
        use windows::win32::com::CoUninitialize;

        unsafe {
            CoUninitialize();
        }
    }
}

fn start() -> windows::Result<()> {
    let _com = InitCom::sta()?;
    let current_file = get_exe_path();

    if create_reg_keys(&current_file) && create_shortcut(&current_file)? {
        let notification = construct_notification()?;
        ToastNotificationManager::history()?.clear_with_id(current_file.clone())?;

        ToastNotificationManager::get_default()?
            .create_toast_notifier_with_id(current_file)?
            .show(notification)?;

        std::thread::sleep(std::time::Duration::from_millis(10));
    } else {
        eprintln!("Failed to create registry keys for: {}", &current_file);
    }

    Ok(())
}

fn create_reg_keys(file: &str) -> bool {
    use std::{mem::transmute, ptr::null_mut};
    use windows::win32::system_services::LSTATUS;
    use windows::win32::windows_programming::{RegCloseKey, RegCreateKeyExW, RegSetValueExA, HKEY};

    let mut result = false;
    let current_user = HKEY(0x80000001);
    let reg_value = 1_u32;
    let subkey = {
        let path = format!(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Notifications\Settings\{}",
            file
        );
        WideString::from_str(&path)
    };
    let mut hkey = HKEY(0);
    let error_success = LSTATUS(0);

    unsafe {
        // Might need before hand? https://docs.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regopencurrentuser
        if RegCreateKeyExW(
            current_user,
            subkey.ptr(),
            0,
            null_mut(),
            0,
            0xF003F,
            null_mut(),
            &mut hkey,
            null_mut(),
        ) == error_success
        {
            result = RegSetValueExA(
                hkey,
                b"ShowInActionCenter\0".as_ptr() as *const i8,
                0,
                4,
                transmute(&reg_value),
                4,
            ) == error_success;
            RegCloseKey(hkey);
        }
    }

    result
}

fn construct_notification() -> windows::Result<ToastNotification> {
    let mut args = env::args();
    let text01 = args.nth(1).unwrap_or("Hello!".to_string());
    let text02 = args.nth(2).unwrap_or_default();

    let notification = {
        let xml = XmlDocument::new()?;
        xml.load_xml(XML_TEMPLATE)?;

        let binding = xml.get_elements_by_tag_name("binding")?.item(0)?;

        let text_nodes = xml.get_elements_by_tag_name("text")?;
        let node1 = text_nodes.item(0)?;
        let node2 = text_nodes.item(1)?;

        node1.set_inner_text(text01)?;
        binding.append_child(node1)?;

        node2.set_inner_text(text02)?;
        binding.append_child(node2)?;

        let root = xml.get_elements_by_tag_name("toast")?.item(0)?;
        let audio = xml.create_element("audio")?;
        audio.set_attribute("src", "ms-winsoundevent:Notification.Default")?;
        root.append_child(audio)?;

        println!("{}", xml.get_xml()?);

        ToastNotification::create_toast_notification(xml)?
    };

    Ok(notification)
}

fn create_shortcut(file: &str) -> windows::Result<bool> {
    use std::ptr::null;
    use windows::win32::com::IPersistFile;
    use windows::win32::shell::{IShellLinkW, ShellLink};

    let mut result = false;

    if let Ok(shell_link) = windows::create_instance::<IShellLinkW>(&ShellLink) {
        let file_path = WideString::from_str(file);

        unsafe {
            let ok1 = shell_link.SetPath(file_path.ptr()).is_ok();
            let ok2 = shell_link.SetDescription(null()).is_ok();

            if ok1 && ok2 {
                if let Ok(persist_file) = shell_link.cast::<IPersistFile>() {
                    let mut shortcut_name = WideString::from_str_with_size(
                        r"E:\Projects\Rust\mini-tools\test.lnk",
                        256,
                    );

                    result = persist_file
                        .Save(shortcut_name.mut_ptr(), true.into())
                        .is_ok();
                }
            }
        }
    }

    Ok(result)
}

/// Gets the filepath for the current executable
/// from std::env::current_exe.
/// If that fails, it assumes the first aurgument
/// of std::env::args is the current exe.
fn get_exe_path() -> String {
    match env::current_exe() {
        Ok(path) => path
            .to_str()
            .unwrap_or(r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\notepad.exe")
            .to_string(),
        Err(_) => env::args().nth(0).unwrap(),
    } //{6D809377-6AF0-444B-8957-A3773F02200E}
}

fn main() {
    let result = start();

    if let Err(err) = result {
        eprintln!("{}", err);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reg() {
        let _com = InitCom::sta().unwrap();
        let exe_path = get_exe_path();
        let reg_status = create_reg_keys(&exe_path);
        let lnk_status = create_shortcut(&exe_path).unwrap();

        dbg!(exe_path);
        dbg!(reg_status);
        dbg!(lnk_status);
    }
}
