# Changelog

## 0.2.3 (2026-11-07)

- **Security:** the start-at-logon and PredatorSense button setups now copy the program to
  `C:\Program Files\Rusty Predator` and run that copy. Before, their tasks ran the program as
  administrator from wherever it was unzipped, where other programs could replace it. Run the
  install scripts again to move existing setups over.
- **Security:** the settings file, `debug.log` and Eco's saved power plan in
  `%APPDATA%\RustyPredator` are now written with the user's own rights, without administrator
  rights. Any program running as the user can redirect that folder, and the button task starts the
  app as administrator without a prompt, so such a program could have had it overwrite system
  files.
- **Security:** `shutdown.exe` and `nvidia-smi.exe` are now found from the Windows folder that
  Windows reports, not from the `%SystemRoot%` variable. Any program of the user can set that
  variable for the button task, which starts the app as administrator without a prompt, so such a
  program could have had the app run its own program as administrator.
- **Fixed:** signing out or shutting down now puts the fans back on Auto and removes Eco's limits,
  as Exit does. Before, Balanced or Eco could leave the fans off with no app to start them.
- **Fixed:** a Turbo button press made while the app was starting, or while its event connection
  was being re-established, was missed until the next press.
- **Removed:** the `turbo_button_events` and `turbo_rearm_probe` research tools, whose results
  are in the firmware notes.

## 0.2.2 (2026-11-06)

- **Changed:** the Turbo button is now reported by the firmware instead of checked once a second.
  The app switches on the firmware's Launch Manager mode at start and after sleep, and off at exit.
  It falls back to checking once a second if the firmware refuses the mode.

## 0.2.1 (2026-11-04)

- **Fixed:** after charging from below 40 % on AC, Turbo and GPU overclock levels stayed refused
  with "Overclocking needs the AC adapter and at least 40% battery" until the next AC change, sleep
  or restart of the app. Choosing one now checks the battery at that moment.

## 0.2.0 (2026-11-03)

- **Renamed to Rusty Predator** (it was Predator Lite). The program is now `rusty-predator.exe`
  and the project is at <https://github.com/yavcsd/rusty-predator>. Settings move to
  `%APPDATA%\RustyPredator` at the first start. To update, see
  [the installation guide](docs/installation.md#from-predator-lite-011-or-older).

## 0.1.1 (2026-10-30)

- **Fixed:** the GPU mode box showed *Discrete only* at start while the laptop ran in Hybrid. The
  firmware reports Discrete after every start on the PT515-51, so the box now shows the mode
  Windows is running in, or the one chosen since Windows started. A switch is always written to
  the firmware, even when it already reports that mode.
- `--status` reports the running GPU mode next to the firmware value.
- **Docs:** on the PT515-51 a full shutdown turns the firmware's Win key lock back on (a
  restart doesn't). Predator Lite turns it off again when it starts and after sleep; the user
  guide now says so and points to starting it at logon.

## 0.1.0 (2026-10-29, first public release)

- **Monitoring:**
  - CPU, GPU and system temperatures.
  - RPM and real duty cycle of all three fans.
- **Fans:**
  - Auto, Max or Custom per group, with Custom calibrated to real duty.
  - CoolBoost.
- **Modes:**
  - Turbo (also from the physical Turbo button).
  - Balanced and Eco, with software fan curves.
  - Eco also turns Turbo Boost off through its own power plan and caps the GPU clock via
    `nvidia-smi`.
- **GPU and display:**
  - GPU overclock levels.
  - GPU mode (Hybrid / Discrete only) with a restart prompt.
  - LCD overdrive.
  - Battery rule for overclocking.
- **Keyboard:**
  - Zone RGB (up to 4 zones), static and the firmware effects.
  - Win key lock.
  - Backlight off when idle.
- **System:**
  - Boot animation and sound.
  - USB charging when off.
- **Buttons and tray:**
  - PredatorSense button: toggles the window, and can start the app with the setup script.
  - Tray icon and menu.
- **Settings:**
  - Saved in `settings.ini` and restored at start and after sleep.
  - Firmware-owned settings are read, never overwritten.
  - The keyboard lighting is left as it is until you change it in the app (the firmware can't
    report its effect).
- **Tools:**
  - Setup scripts for start at logon and the PredatorSense button.
  - Read-only `--status` report.
