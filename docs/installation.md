# Installation

- [Requirements](#requirements)
- [Install](#install)
- [Coming from PredatorSense](#coming-from-predatorsense)
- [First start](#first-start)
- [Start at logon](#start-at-logon)
- [Running the setup scripts](#running-the-setup-scripts)
- [Updating](#updating)
- [Uninstalling](#uninstalling)

## Requirements

- **Windows 10 or 11, 64-bit.**
- **An Acer laptop whose BIOS offers Acer's gaming WMI interface.** That is most Predator, Helios,
  Triton and Nitro models from about 2018 to 2021. Newer models are not supported yet. See
  [Supported laptops](../README.md#supported-laptops).
- **Windows 10 version 1809 or later** for the optional PredatorSense button setup (it starts the
  app through `conhost --headless`).
- **Administrator rights.** The BIOS methods only answer administrators, so the app asks for
  elevation when it starts.
- **Optional:** the NVIDIA driver, which installs `nvidia-smi`. Without it, Eco mode still works
  but can't cap the GPU clock.

Nothing else is needed: no .NET, no Visual C++ redistributable, no driver.

## Install

1. Download `rusty-predator.zip` from the
   [Releases page](https://github.com/yavcsd/rusty-predator/releases).
2. Unzip it into a folder of your choice, for example `C:\Tools\Rusty Predator`. The optional
   start-at-logon and button setups copy `rusty-predator.exe` to `C:\Program Files\Rusty Predator`
   and run that copy, so moving the folder afterwards doesn't break them.

The folder holds:

| File | What it is |
|---|---|
| `rusty-predator.exe` | The program |
| `README.md`, `LICENSE` | Overview and license. The full guides are on the [project page](https://github.com/yavcsd/rusty-predator#guides). |
| `tools\install-autostart.ps1`, `tools\uninstall-autostart.ps1` | [Start at logon](#start-at-logon) |
| `tools\install-predatorsense-button.ps1`, `tools\uninstall-predatorsense-button.ps1` | [PredatorSense button setup](predatorsense-button.md) |
| `tools\common.ps1` | Shared by the scripts above |

If you'd rather build it yourself, see [Building from source](../README.md#building-from-source).
The setup scripts then find the program in `target\release` automatically.

## Coming from PredatorSense

PredatorSense and Rusty Predator both control the same fans, LEDs and settings, and running both
makes them fight. Its background service can, for example, put its own fan mode back after you
change it. Either:

- **Uninstall PredatorSense** (Settings > Apps > Installed apps > PredatorSense). This is the
  cleanest option. Remove any separately listed PredatorSense service package too.
- **Or keep it but don't run it.** Exit PredatorSense, then in `services.msc` find its service
  (its program is `PSSvc.exe`; the display name varies by version) and set it to *Disabled*.

Your firmware settings (keyboard colors, GPU mode, …) stay where PredatorSense left them.
Rusty Predator reads them at its first start. The keyboard effect can't be read, so the lighting
is left untouched until you change it: see [First start](#first-start).

## First start

Double-click `rusty-predator.exe` and accept the administrator prompt. The program isn't
code-signed, so the first time Windows SmartScreen may show "Windows protected your PC": click
**More info**, then **Run anyway**.

- The window opens and a Predator icon appears in the notification area (tray). Windows may put
  it in the hidden-icons overflow (the `^` arrow). Drag it next to the clock to keep it visible.
- **The first start only reads.** It shows what the firmware currently has and changes nothing.
  From your first change on, your choices are saved to `%APPDATA%\RustyPredator\settings.ini` and
  applied again at every start and after sleep.
- **Keyboard effect:** the firmware reports the zone colors but not the current effect, so the
  window shows **Static** even if PredatorSense left an animated effect (Wave, Breathing, …).
  Rusty Predator leaves the keyboard lighting exactly as it is until you change the lighting
  yourself in the window. From then on, your lighting is restored at every start.
- Closing the window (×) hides it to the tray, so the hardware buttons and fan curves keep
  working. To quit, right-click the tray icon and choose **Exit**. Exiting from Balanced or Eco
  puts the fans back on the firmware's Auto and removes Eco's limits; your mode comes back at
  the next start. Other fan settings stay as they are, since the firmware keeps them.

Next: the [user guide](user-guide.md) walks through every control.

## Start at logon

Rusty Predator has to be running for these to work:
- the Turbo button's fans and LED,
- the Balanced and Eco fan curves,
- restoring your settings after sleep.

To start it in the tray at every logon (without a UAC prompt), run this in an **administrator**
PowerShell in the Rusty Predator folder (see [Running the setup scripts](#running-the-setup-scripts)):

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\install-autostart.ps1
```

This copies `rusty-predator.exe` to `C:\Program Files\Rusty Predator` and registers a scheduled
task named **Rusty Predator** that runs that copy with `--hidden` at logon, with administrator
rights. It may start on battery, keeps running when unplugged, and has no time limit, unlike
Windows' defaults for scheduled tasks.

The task runs the copy, not the file you unzipped: it starts with administrator rights and no
prompt, so it must sit where only administrators can write. Otherwise any program could replace
it and get administrator rights without asking.

To undo it:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\uninstall-autostart.ps1
```

## Running the setup scripts

The scripts in `tools\` change system settings (scheduled tasks, a keyboard remap), so they must
run in an **administrator** PowerShell:

1. Open Start, type `PowerShell`, right-click **Windows PowerShell** and choose **Run as
   administrator**.
2. Go to the Rusty Predator folder, e.g. `cd "C:\Tools\Rusty Predator"`.
3. Run the script with `powershell -ExecutionPolicy Bypass -File .\tools\<script>.ps1`.
   `-ExecutionPolicy Bypass` applies to that one run only. It doesn't change your system's
   script policy.

If `rusty-predator.exe` is somewhere else, pass its path: `... -File .\tools\install-autostart.ps1
-Exe "D:\Apps\rusty-predator.exe"`. The install scripts copy it to `C:\Program Files\Rusty Predator`.

## Updating

1. Right-click the tray icon and choose **Exit**.
2. Replace `rusty-predator.exe` with the new one, in the same folder.
3. If you use [start at logon](#start-at-logon) or the
   [PredatorSense button setup](predatorsense-button.md), run one of their install scripts again.
   It copies the new version to `C:\Program Files\Rusty Predator`, which both setups use.
4. Start it again, or log off and on if you use start at logon.

Settings files from pre-release builds are read and converted automatically. That includes fan
percentages saved before the duty calibration and the old "silent" / "quiet" mode names.

### From Predator Lite (0.1.1 or older)

The app used to be called Predator Lite, and the program file now has a new name:

1. Right-click the tray icon and choose **Exit**, then delete `predator-lite.exe`.
2. Put `rusty-predator.exe` and the `tools` folder in its place.
3. If you use [start at logon](#start-at-logon) or the
   [PredatorSense button setup](predatorsense-button.md), run its install script again. It
   replaces the old scheduled task and shortcut.

Your settings carry over: the first start moves `%APPDATA%\PredatorLite` to
`%APPDATA%\RustyPredator`, and Eco renames its power plan the next time it runs.

## Uninstalling

1. Undo the optional setups, if you used them:
   ```powershell
   powershell -ExecutionPolicy Bypass -File .\tools\uninstall-autostart.ps1
   powershell -ExecutionPolicy Bypass -File .\tools\uninstall-predatorsense-button.ps1
   ```
   The button removal needs a restart to give the button back its original scan code. Once both
   are undone, the copy in `C:\Program Files\Rusty Predator` is deleted too.
2. Set both fan groups to **Auto** if they aren't, then right-click the tray icon and choose
   **Exit**. The firmware keeps the last fan mode after the app is gone.
3. Delete the Rusty Predator folder and `%APPDATA%\RustyPredator`, which holds `settings.ini`,
   `eco_prev_plan` while Eco is on, and the optional debug log.
4. If you ever used Eco mode, remove its power plan.
   - **Control Panel:** Control Panel > Power Options, select another plan first, then delete
     **Rusty Predator Eco** (or **Predator Lite Eco**, if you last used Eco before the rename).
   - **Command line:** `powercfg /list` to find its GUID, then `powercfg /delete <GUID>`.

Firmware settings (keyboard colors, GPU mode, …) stay as you last set them. They live in the
laptop, not in Windows. The Win key is the exception: on the PT515-51 a full shutdown locks it
again, until something turns it back on.
