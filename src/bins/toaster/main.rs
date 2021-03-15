#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[path = "../../com_helper.rs"]
mod com_helper;
#[path = "../../wstring.rs"]
mod wstring;

use clap::ArgMatches;
use com_helper::InitCom;
use windows::win32::system_services::LSTATUS;
use windows::Interface;
use windows::{
    data::xml::dom::XmlDocument,
    ui::notifications::{ToastNotification, ToastNotificationManager},
};
use winrt::windows;
use wstring::WideString;

use std::{env, path::Path};

const ERROR_SUCCESS: LSTATUS = LSTATUS(0);
const LAME_SUN: &[u8] = include_bytes!("../../../assets/lame_sun.png");

fn start(input: &clap::ArgMatches) -> windows::Result<()> {
    let _com = InitCom::sta()?;
    let mut current_file = get_exe_path();

    let mut shortcut = env::var("APPDATA").unwrap();
    shortcut.push_str(r"\Microsoft\Windows\Start Menu\Programs\Toaster.lnk");

    if let Some(app_id) = input.value_of("AppID") {
        current_file = app_id.to_string();
    } else {
        if let None = input.value_of("NoShortcut") {
            create_shortcut(&current_file, &shortcut)?;
        }
    }

    if create_reg_keys(&current_file) {
        let notification = construct_notification(input)?;

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
        ) == ERROR_SUCCESS
        {
            result = RegSetValueExA(
                hkey,
                b"ShowInActionCenter\0".as_ptr().cast(),
                0,
                4,
                transmute(&reg_value),
                4,
            ) == ERROR_SUCCESS;
            RegCloseKey(hkey);
        }
    }

    result
}

fn construct_notification(input: &ArgMatches) -> windows::Result<ToastNotification> {
    use std::fs;
    let image_tag;

    if let Some(image_path) = input.value_of("IconPath") {
        if Path::new(image_path).exists() {
            image_tag = format!(
                "<image placement=\"appLogoOverride\" hint-crop=\"none\" src=\"{}\"/>",
                image_path
            );
        } else {
            image_tag = String::new();
        }
    } else {
        let mut defualt_image_path = env::var("TEMP").unwrap();
        defualt_image_path.push_str("toast_default.png");

        fs::write(&defualt_image_path, LAME_SUN).unwrap_or_else(|_err| {
            eprintln!("Failed to save template image as {}", &defualt_image_path);
        });

        image_tag = format!(
            "<image placement=\"appLogoOverride\" hint-crop=\"none\" src=\"{}\"/>",
            defualt_image_path
        );
    }

    let text01 = match input.value_of("Headline") {
        Some(s) => s,
        None => "Hello!",
    };
    let text02 = match input.value_of("Text") {
        Some(s) => s,
        None => "",
    };

    let notification = {
        let xml = XmlDocument::new()?;
        let xml_text = format!(
            r#"
<toast scenario="reminder" launch="developer-pre-defined-string">
  <visual>
    <binding template="ToastGeneric">
      {}
      <text>{}</text>
      <text>{}</text>      
      <text placement="attribution">{}</text>
    </binding>
  </visual>
  <audio src= "ms-winsoundevent:Notification.Default"/>
</toast>"#,
            image_tag, text01, text02, ""
        );

        xml.load_xml(xml_text)?;
        ToastNotification::create_toast_notification(xml)
    };

    notification
}

fn get_cli_inputs() -> clap::ArgMatches<'static> {
    use clap::clap_app;
    clap_app!(Toaster =>
        (version: "1.0")
        (about: "An easy way to display Toast notifications\n\nNOTE: It can fail the first time, if the start menu is not updated")
        (@arg Headline: -h --headline +takes_value "Sets the headline of the Toast")
        (@arg Text: -t --text +takes_value "Sets the message of the Toast")
        (@arg IconPath: -i --icon +takes_value "Sets the icon of the Toast")
        (@arg File: -f --file +takes_value "Uses an XML file as the Toast")
        (@arg NoShortcut: -n --noshortcut "Toaster doesn't try to create it's a shortcut in start menu (userspace)")
        (@arg AppID: -a --appid +takes_value "The location the program you want to display the message. The program must have a shortcut in the start menu")
    ).get_matches()
}

fn create_shortcut(file: &str, shortcut: &str) -> windows::Result<bool> {
    use windows::win32::com::IPersistFile;
    use windows::win32::shell::{IShellLinkW, ShellLink};

    let mut result = false;

    if let Ok(shell_link) = windows::create_instance::<IShellLinkW>(&ShellLink) {
        let file_path = WideString::from_str(file);
        let folder = WideString::from_str(Path::new(file).parent().unwrap().to_str().unwrap());

        unsafe {
            let ok1 = shell_link.SetPath(file_path.ptr()).is_ok();
            let ok2 = shell_link.SetWorkingDirectory(folder.ptr()).is_ok();
            // let ok3 = shell_link.SetIconLocation(psz_icon_path, i_icon)

            if ok1 && ok2 {
                // shell_link.cast == IShellLink::QueryInterface
                if let Ok(persist_file) = shell_link.cast::<IPersistFile>() {
                    let mut shortcut_name = WideString::from_str_with_size(&shortcut, 256);

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
            .unwrap_or(r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\cmd.exe")
            .to_string(),
        Err(_) => env::args().nth(0).unwrap(),
    } //{6D809377-6AF0-444B-8957-A3773F02200E}
}

fn main() {
    let args = get_cli_inputs();
    let result = start(&args);

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
        let mut shortcut = env::var("APPDATA").unwrap();
        shortcut.push_str(r"\Microsoft\Windows\Start Menu\Programs\Toaster.lnk");
        let exe_path = get_exe_path();
        let reg_status = create_reg_keys(&exe_path);
        let lnk_status = create_shortcut(&exe_path, &shortcut).unwrap();

        dbg!(exe_path);
        dbg!(reg_status);
        dbg!(lnk_status);
    }
}
