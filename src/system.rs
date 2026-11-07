//! Small Windows helpers shared by the window, Eco's limits and the example tools: starting
//! System32 programs without a console window, reading the power status, listing the display
//! adapters, remembering a GPU mode switch until the next restart, writing user files without
//! administrator rights, and struct sizes for Win32 calls.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use windows::core::{w, Owned, PCWSTR};
use windows::Win32::Foundation::{ERROR_SUCCESS, HANDLE};
use windows::Win32::Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW};
use windows::Win32::Security::{
    CreateRestrictedToken, CreateWellKnownSid, GetTokenInformation, ImpersonateLoggedOnUser, RevertToSelf,
    SetTokenInformation, TokenOwner, TokenUser, WinBuiltinAdministratorsSid, DISABLE_MAX_PRIVILEGE, PSID,
    SID_AND_ATTRIBUTES, TOKEN_ADJUST_DEFAULT, TOKEN_DUPLICATE, TOKEN_OWNER, TOKEN_QUERY, TOKEN_USER,
};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegGetValueW, RegSetValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_SET_VALUE, REG_DWORD,
    REG_OPTION_VOLATILE, RRF_RT_REG_DWORD,
};
use windows::Win32::System::SystemInformation::GetSystemDirectoryW;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// Process creation flag: no console window for the child.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// Volatile registry key (Windows drops it at the next restart) holding the GPU mode chosen in
/// this Windows session. HKLM, like the firmware setting it mirrors: it is machine-wide and a
/// sign-out must not lose it.
const PENDING_GPU_MODE_KEY: PCWSTR = w!("SOFTWARE\\RustyPredator");
const PENDING_GPU_MODE_VALUE: PCWSTR = w!("PendingGpuMode");
/// `SYSTEM_POWER_STATUS::BatteryFlag`: no system battery.
const BATTERY_FLAG_NONE: u8 = 128;

/// `size_of::<T>()` as the `u32` that Win32 size fields (`cbSize`, `dwSize`, ...) take.
#[must_use]
#[allow(clippy::cast_possible_truncation, reason = "Win32 structs are a few hundred bytes")]
pub const fn size_u32<T>() -> u32 {
    size_of::<T>() as u32
}

/// Path of a program in the System32 folder.
///
/// The folder comes from Windows, not from `%SystemRoot%`: any program of the user can set that
/// variable (`HKCU\Environment`) for the scheduled tasks that start this program as administrator.
#[must_use]
pub fn system32_exe(name: &str) -> PathBuf {
    let mut buf = [0u16; 260]; // MAX_PATH
    let dir = match unsafe { GetSystemDirectoryW(Some(&mut buf)) } {
        n @ 1..260 => PathBuf::from(OsString::from_wide(&buf[..n as usize])),
        _ => PathBuf::from(r"C:\Windows\System32"),
    };
    dir.join(name)
}

/// Run `f` as the user without the Administrators group and without privileges, as an
/// unelevated program of the user would run.
///
/// For files in the user's folders (`%APPDATA%\RustyPredator`): the user, and any program
/// running as the user, controls those paths and `%APPDATA%` itself, and can turn them into links
/// to system files (a junction to `\RPC Control` plus object-manager symlinks). The scheduled
/// tasks start this program as administrator without a prompt, so its own rights must not be
/// used there: as the plain user, a link only reaches files that user can already change.
///
/// # Errors
/// When the restricted token can't be made or used (`f` is not run then), or `f`'s error.
pub fn as_plain_user<T>(f: impl FnOnce() -> std::io::Result<T>) -> std::io::Result<T> {
    let token = plain_user_token()?;
    unsafe { ImpersonateLoggedOnUser(*token)? };
    let result = f();
    // A failed revert would leave this thread with fewer rights, never more.
    let _ = unsafe { RevertToSelf() };
    result
}

/// This process's token with the Administrators group deny-only and all privileges removed.
fn plain_user_token() -> windows::core::Result<Owned<HANDLE>> {
    unsafe {
        let mut process = Owned::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ADJUST_DEFAULT, &raw mut *process)?;
        let mut admins = [0u32; 17]; // SECURITY_MAX_SID_SIZE bytes, aligned for a SID
        let admins_sid = PSID(admins.as_mut_ptr().cast());
        let mut len = size_u32::<[u32; 17]>();
        CreateWellKnownSid(WinBuiltinAdministratorsSid, None, Some(admins_sid), &raw mut len)?;
        let disable = [SID_AND_ATTRIBUTES { Sid: admins_sid, Attributes: 0 }];
        let mut token = Owned::default();
        CreateRestrictedToken(*process, DISABLE_MAX_PRIVILEGE, Some(&disable), None, None, &raw mut *token)?;
        // An elevated token's default owner is the Administrators group, which can't own new
        // files once deny-only: new files are owned by the user instead.
        let mut user = [0u64; 32]; // TOKEN_USER and the SID it points to, aligned for TOKEN_USER
        let mut len = 0;
        GetTokenInformation(*token, TokenUser, Some(user.as_mut_ptr().cast()), size_u32::<[u64; 32]>(), &raw mut len)?;
        let owner = TOKEN_OWNER { Owner: user.as_ptr().cast::<TOKEN_USER>().read().User.Sid };
        SetTokenInformation(*token, TokenOwner, (&raw const owner).cast(), size_u32::<TOKEN_OWNER>())?;
        Ok(token)
    }
}

/// A command for a System32 program that runs without a console window.
#[must_use]
pub fn hidden_command(exe_name: &str) -> Command {
    let mut command = Command::new(system32_exe(exe_name));
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

/// (on AC, battery percent if known), for `PowerGate::update`.
#[must_use]
pub fn power_status() -> (bool, Option<u8>) {
    let mut ps = SYSTEM_POWER_STATUS::default();
    if unsafe { GetSystemPowerStatus(&raw mut ps) }.is_err() {
        return (true, None);
    }
    // ACLineStatus 255 = unknown: assume AC so a desktop-like setup is not locked out.
    let on_ac = ps.ACLineStatus != 0;
    let known = ps.BatteryLifePercent <= 100 && ps.BatteryFlag & BATTERY_FLAG_NONE == 0;
    (on_ac, known.then_some(ps.BatteryLifePercent))
}

/// Device ids of the display adapters Windows sees (e.g. `PCI\VEN_8086&DEV_3E9B&...`), for
/// `protocol::running_gpu_mode`.
#[must_use]
pub fn display_adapter_ids() -> Vec<String> {
    let mut ids = Vec::new();
    for i in 0..16 {
        let mut dd = DISPLAY_DEVICEW { cb: size_u32::<DISPLAY_DEVICEW>(), ..Default::default() };
        if !unsafe { EnumDisplayDevicesW(PCWSTR::null(), i, &raw mut dd, 0) }.as_bool() {
            break;
        }
        let n = dd.DeviceID.iter().position(|&c| c == 0).unwrap_or(dd.DeviceID.len());
        ids.push(String::from_utf16_lossy(&dd.DeviceID[..n]));
    }
    ids
}

/// The GPU mode value (`GpuMode as u8`) chosen since Windows started, if any. The firmware's
/// own value can't tell: on the PT515-51 it reads Discrete after every start.
#[must_use]
pub fn pending_gpu_mode() -> Option<u8> {
    read_dword(HKEY_LOCAL_MACHINE, PENDING_GPU_MODE_KEY, PENDING_GPU_MODE_VALUE).and_then(|v| u8::try_from(v).ok())
}

/// Remember a GPU mode switch until the next restart (needs administrator rights).
///
/// # Errors
/// The registry error, if the key can't be created or written.
pub fn remember_pending_gpu_mode(mode: u8) -> windows::core::Result<()> {
    write_volatile_dword(HKEY_LOCAL_MACHINE, PENDING_GPU_MODE_KEY, PENDING_GPU_MODE_VALUE, mode.into())
}

/// Write a DWORD, creating `subkey` as a volatile key if it doesn't exist.
fn write_volatile_dword(root: HKEY, subkey: PCWSTR, name: PCWSTR, v: u32) -> windows::core::Result<()> {
    let mut key = HKEY::default();
    unsafe {
        let r = RegCreateKeyExW(
            root,
            subkey,
            None,
            PCWSTR::null(),
            REG_OPTION_VOLATILE,
            KEY_SET_VALUE,
            None,
            &raw mut key,
            None,
        );
        r.ok()?;
        let r = RegSetValueExW(key, name, None, REG_DWORD, Some(&v.to_le_bytes()));
        let _ = RegCloseKey(key);
        r.ok()
    }
}

/// A DWORD value, or `None` if the key or value is missing or not a DWORD.
fn read_dword(root: HKEY, subkey: PCWSTR, name: PCWSTR) -> Option<u32> {
    let mut v = 0u32;
    let mut size = size_u32::<u32>();
    let r = unsafe {
        RegGetValueW(root, subkey, name, RRF_RT_REG_DWORD, None, Some((&raw mut v).cast()), Some(&raw mut size))
    };
    (r == ERROR_SUCCESS).then_some(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_u32_matches_size_of() {
        assert_eq!(size_u32::<u64>(), 8);
        assert_eq!(usize::try_from(size_u32::<SYSTEM_POWER_STATUS>()), Ok(size_of::<SYSTEM_POWER_STATUS>()));
    }

    #[test]
    fn system32_exe_is_under_system32() {
        let path = system32_exe("cmd.exe");
        // Windows reports the folder as `system32`.
        assert!(path.to_string_lossy().to_lowercase().ends_with(r"\system32\cmd.exe"), "{path:?}");
        assert!(path.is_absolute(), "{path:?}");
        assert_eq!(system32_exe("nvidia-smi.exe").parent(), path.parent());
        assert!(path.exists(), "{path:?}");
        // A user-set %SystemRoot% must not move it.
        let saved = std::env::var_os("SystemRoot");
        std::env::set_var("SystemRoot", r"C:\NotWindows");
        let moved = system32_exe("cmd.exe");
        std::env::set_var("SystemRoot", saved.unwrap_or_default());
        assert_eq!(moved, path);
    }

    #[test]
    fn hidden_command_runs_the_system32_program() {
        let command = hidden_command("shutdown.exe");
        assert_eq!(std::path::Path::new(command.get_program()), system32_exe("shutdown.exe"));
        assert_eq!(command.get_args().count(), 0);
    }

    #[test]
    fn power_status_is_consistent() {
        let (_, pct) = power_status();
        assert!(pct.is_none_or(|p| p <= 100), "{pct:?}");
    }

    #[test]
    fn display_adapters_are_listed() {
        // A CI machine may have no display adapter, so only the ids' form is checked.
        let ids = display_adapter_ids();
        assert!(ids.len() <= 16, "{ids:?}");
        assert!(ids.iter().all(|id| !id.contains('\0')), "{ids:?}");
    }

    #[test]
    fn volatile_dword_round_trip() {
        use windows::core::HSTRING;
        use windows::Win32::System::Registry::{RegDeleteKeyW, HKEY_CURRENT_USER};
        // HKCU so the test needs no administrator rights; the key is removed afterwards.
        let subkey = HSTRING::from(format!("Software\\RustyPredatorTest{}", std::process::id()));
        let subkey = PCWSTR(subkey.as_ptr());
        let name = w!("Value");
        assert_eq!(read_dword(HKEY_CURRENT_USER, subkey, name), None);
        write_volatile_dword(HKEY_CURRENT_USER, subkey, name, 2).unwrap();
        assert_eq!(read_dword(HKEY_CURRENT_USER, subkey, name), Some(2));
        write_volatile_dword(HKEY_CURRENT_USER, subkey, name, 1).unwrap();
        assert_eq!(read_dword(HKEY_CURRENT_USER, subkey, name), Some(1));
        assert_eq!(read_dword(HKEY_CURRENT_USER, subkey, w!("Missing")), None);
        assert_eq!(unsafe { RegDeleteKeyW(HKEY_CURRENT_USER, subkey) }, ERROR_SUCCESS);
        assert_eq!(read_dword(HKEY_CURRENT_USER, subkey, name), None);
    }

    #[test]
    fn as_plain_user_drops_the_administrators_group() {
        use windows::Win32::Security::CheckTokenMembership;
        use windows::Win32::System::Threading::{GetCurrentThread, OpenThreadToken};
        let is_admin = || {
            let mut sid = [0u32; 17];
            let sid = PSID(sid.as_mut_ptr().cast());
            let mut len = size_u32::<[u32; 17]>();
            let mut member = windows::core::BOOL(1);
            unsafe {
                CreateWellKnownSid(WinBuiltinAdministratorsSid, None, Some(sid), &raw mut len).unwrap();
                CheckTokenMembership(None, sid, &raw mut member).unwrap();
            }
            member.as_bool()
        };
        // Only meaningful when the tests run elevated; unelevated the group is deny-only already.
        let file = std::env::temp_dir().join(format!("RustyPredatorPlainUser{}", std::process::id()));
        let inside = as_plain_user(|| {
            std::fs::write(&file, "x")?;
            Ok(is_admin())
        });
        assert_eq!(inside.as_ref().ok(), Some(&false), "{inside:?}");
        assert_eq!(std::fs::read_to_string(&file).ok().as_deref(), Some("x"));
        std::fs::remove_file(&file).unwrap();
        // The thread is back on the process token.
        let mut thread_token = HANDLE::default();
        assert!(unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, true, &raw mut thread_token) }.is_err());
        assert!(as_plain_user(|| -> std::io::Result<()> { Err(std::io::ErrorKind::Other.into()) }).is_err());
    }
}
