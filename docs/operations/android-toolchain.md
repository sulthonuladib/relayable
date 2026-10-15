# Android / JDK / Gradle toolchain audit — ENV-001 (read-only)

**Scope:** read-only inventory of the Android CLI, `adb`, Android SDK, emulator/AVDs, KVM,
JDK/mise selection, Gradle availability, and Android Studio on this host, per `full.md`
§0.4, §10.1, §13.5 and task ledger row ENV-001 / AND-001.

**Method:** every command below is read-only. No installs, `sudo`, downloads, license
acceptance, service/port changes, builds, or writes outside this file were performed. Commands
that could mutate user/system state (`android init`, `android sdk install/remove/update`,
`android emulator create/start/remove`, `android skills add/remove/update`, `android auth`,
`android update`, `gradle`) were deliberately **not** run; only their `-h` text is recorded.
`android sdk list` was also not run: it may fetch/update repository metadata (state write).

**Remediation provenance (2026-10-15, ~23:18 +07):** the 36 rows flagged as missing exit codes
in the independent ENV-001 verification were re-captured in a follow-up read-only pass on
2026-10-15 by re-running the same commands. Outputs matched, with these divergences: the
header `date -Is` (re-run `2026-10-15T23:18:07+07:00`) and `git rev-parse HEAD` (re-run
`1b62c50a44dc8e3e3263dee05c01a0574a168b21`) are time/HEAD dependent, so the audit-time values
are retained in the header block below with their re-captured exit status; B1.5's listings are
now raw remediation-time output (the original was elided); and the B3 `find … | head -20`
re-run emitted two stderr `Permission denied` lines that were absent from the original record
while stdout stayed empty (`find` rc=1, `head` rc=0). No `android` CLI command was re-run, so
no CLI bookkeeping timestamps changed (`last_update_check`/`last_upload_time` remain
`2026-10-15 23:05:15 +07`).

## Header context (applies to every row below)

| Field | Value |
|---|---|
| Worktree / cwd | `/home/sulthonuladib/.herdr/worktrees/relayable/research-env-audit` |
| Branch | `research/env-audit` |
| HEAD at audit time | `3db73c2d28c6b7a98e424456768299e8d51b694d` |
| `git status --short --branch` (before) | `## research/env-audit` (clean, no entries) |
| Date/time of audit | 2026-10-15, ~23:05–23:10 +07 (`date -Is` → `2026-10-15T23:05:14+07:00`) |
| Hostname | `falcon` |
| OS / kernel | `Linux falcon 7.2.5-3-omarchy #1 SMP PREEMPT_DYNAMIC Mon, 14 Sep 2026 19:55:01 +0000 x86_64 GNU/Linux` |
| User | `sulthonuladib` (uid 1000) |
| Shell scope | `bash` (non-interactive tool calls) |

Raw header commands:

```
$ pwd
/home/sulthonuladib/.herdr/worktrees/relayable/research-env-audit
exit=0

$ git status --short --branch
## research/env-audit
exit=0

$ git rev-parse HEAD
3db73c2d28c6b7a98e424456768299e8d51b694d
exit=0            # re-captured; re-run returned 1b62c50a44dc8e3e3263dee05c01a0574a168b21

$ date -Is
2026-10-15T23:05:14+07:00
exit=0            # re-captured; re-run returned 2026-10-15T23:18:07+07:00

$ hostname
falcon
exit=0

$ uname -a
Linux falcon 7.2.5-3-omarchy #1 SMP PREEMPT_DYNAMIC Mon, 14 Sep 2026 19:55:01 +0000 x86_64 GNU/Linux
exit=0

$ whoami
sulthonuladib
exit=0
```

---

## B1 — Android CLI

Captured 2026-10-15 ~23:05–23:08 +07 on `falcon` (worktree above).

### B1.1 Version and binary location

```
$ command -v android
/home/sulthonuladib/.local/bin/android
exit=0

$ android -V
1.0.16500706
exit=0

$ ls -l /home/sulthonuladib/.local/bin/android
-rwx--x--x 1 sulthonuladib sulthonuladib 5041744 Oct 14 20:41 /home/sulthonuladib/.local/bin/android
exit=0

$ readlink -f /home/sulthonuladib/.local/bin/android
/home/sulthonuladib/.local/bin/android
exit=0

$ file "$(readlink -f /home/sulthonuladib/.local/bin/android)"
/home/sulthonuladib/.local/bin/android: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, for GNU/Linux 2.6.24, not stripped
exit=0
```

### B1.2 `android -h` (full help text; this is how subcommands/flags were discovered)

```
$ android -h
Usage: android [-Vhv] [--sdk=PARAM] [COMMAND]
  -h, --help       Shows the help message for the specified command
      --sdk=PARAM  Path to the Android SDK
  -v, --verbose    Enable verbose output for troubleshooting
  -V, --version    Print version information and exit
Commands:
  auth        Authentication commands. Log in/out of Google services for Android CLI
  completion  Installs shell autocomplete configuration for Android CLI in the current user profile
  create      Creates a new Android project from available templates. You can specify the
              project name, output directory, `minSdk` value, and dry-run execution
  describe    Analyzes an Android project to generate descriptive metadata. This command
              identifies and outputs the paths to JSON files that detail the project's
              structure, including build targets and their corresponding output artifact
              locations (such as APKs). This information enables other tools and commands to
              locate build artifacts efficiently
  device      Manage physical Android devices. Create remote physical device reservations
  docs        Searches and fetches developer documentation from the official Android Knowledge Base
  emulator    Manages Android Virtual Devices (AVDs). Includes commands to start, stop, list,
              and view details about emulators
  help        Shows the help information for a specified command
  info        Prints environment information including SDK location, connected devices, and
              configuration variables
  init        Initializes the environment for Android CLI. Sets up required configurations,
              directories, and default skills
  install     Installs an Android app (one or more APKs) on a connected device or emulator
              without activating any components, using incremental optimizations for faster
              deployment than `adb`
  layout      Returns the layout tree of an app
  run         Builds, deploys, and launches an Android app on a connected device or emulator
  screen      Captures and inspects the screen of a connected Android device or emulator
  sdk         Manages the Android SDK installation. Includes commands to install, update,
              remove, and list available and installed SDK packages
  skills      Manages Android CLI skills. Includes commands to install, remove, list, and
              search for skills by keyword
  studio      Connects Android CLI to a running Android Studio instance to analyze files,
              find declarations and usages, render Compose previews, and look up library versions
  update      Updates Android CLI to the latest version
exit=0
```

### B1.3 `android info` — read-only environment info

```
$ android info -h
Usage: android info [-h] [<field>]
Prints environment information including SDK location, connected devices, and configuration variables

Options:
  -h, --help       Shows the help message for the specified command
android options:
      --sdk=PARAM  Path to the Android SDK
  -v, --verbose    Enable verbose output for troubleshooting
  -V, --version    Print version information and exit
Positional Parameters:
  <field>  The specific field to print the value of. If omitted, prints all fields
exit=0

$ android info
sdk: /home/sulthonuladib/Android/Sdk
version: 1.0.16500706
launcher_version: 1.0.16500706
exit=0
```

**Finding:** the CLI's configured/default SDK path is `/home/sulthonuladib/Android/Sdk`, but
that directory does **not exist** (see B3). `ANDROID_HOME` and `ANDROID_SDK_ROOT` are unset.

### B1.4 Relevant read-only subcommand help probes (no execution)

```
$ android sdk -h
Usage: android sdk [-h] [--ignore-outdated-xmls] [--platform=PARAM] [COMMAND]
Commands:
  install  Installs SDK packages
  list     Lists installed and available SDK packages
  remove   Removes packages from the SDK
  update   Updates one or all packages to the latest version
exit=0

$ android sdk list -h
Usage: android sdk list [-h] [--all] [--all-versions] [--beta] [--canary] [<pattern>]
Options:
      --all                   Show all packages available in the repository
      --all-versions          Show all versions for each package
      --beta                  Include beta packages
      --canary                Include canary packages
exit=0

$ android emulator -h
Usage: android emulator [-h] [COMMAND]
Commands:
  create  Creates a virtual device
  list    Lists available virtual devices
  remove  Deletes a virtual device
  start   Launches the specified virtual device. This command returns when the emulator is
          fully started and ready to use
  stop    Stops the specified virtual device
exit=0

$ android emulator list -h
Usage: android emulator list [-h] [--long]
Lists available virtual devices
Options:
  -h, --help       Shows the help message for the specified command
      --long       Give more detailed information
exit=0

$ android device -h
Usage: android device [-h] [COMMAND]
Commands:
  remote  Manage remote physical devices. Includes commands to list, reserve,
          connect to, and manage remote device reservations
exit=0

$ android create -h
Usage: android create [-h] [--application-id=PARAM] [--force] [--list] [--min-sdk=PARAM]
                      [--name=PARAM] [--namespace=PARAM] [--output=PARAM] [<template-name>]
Options:
      --application-id=PARAM  The app ID for the app, for example `com.example.myapp`
      --force                 Allow creating the project in a non-empty directory, overwriting existing files
  -h, --help                  Shows the help message for the specified command
      --list                  List all available templates
      --min-sdk=PARAM         The `minSdk` value supported by the app (the default value is defined in the template)
      --name=PARAM            The name of the app, for example `My Application`
      --namespace=PARAM       The namespace for resources and package name for Kotlin source files
  -o, --output=PARAM          The destination project directory path (the default value is `.`)
exit=0

$ android run -h
Usage: android run [-h] [--activity=PARAM] [--apks=PARAM] [--debug] [--device=PARAM]
                   [--install-options=PARAM] [--type=PARAM] [--use-delta-install]
Builds, deploys, and launches an Android app on a connected device or emulator
exit=0

$ android install -h
Usage: android install [-h] [--apks=PARAM] [--device=PARAM] [--install-options=PARAM]
                       [--use-delta-install]
Installs an Android app (one or more APKs) on a connected device or emulator
without activating any components, using incremental optimizations for faster deployment than `adb`
exit=0

$ android skills -h
Usage: android skills [-h] [COMMAND]
Commands:
  add     Installs a specific skill by its ID to your environment
  find    Searches for available skills in the repository matching a keyword
  list    Lists installed and available skills
  remove  Removes an installed skill by its ID
  update  Updates installed skills
exit=0
```

Not probed (avoided because they are mutating or out of scope here): `android init`,
`android sdk install/remove/update`, `android emulator create/start/remove`,
`android skills add/remove/update`, `android auth`, `completion`, `update`.

### B1.5 Pre-existing Android CLI state (read-only observation)

`android init` had already been run on this host before this audit (bundled skills and
analytics state exist under `~/.android/cli`). This audit did not run `init` and did not
modify those directories.

```
$ ls -la ~/.android/cli
total 8
drwxr-xr-x 1 sulthonuladib sulthonuladib 104 Oct 14 20:44 .
drwxr-x--- 1 sulthonuladib sulthonuladib  96 Oct 14 20:42 ..
drwxr-xr-x 1 sulthonuladib sulthonuladib  46 Oct 14 20:42 analytics
drwxr-xr-x 1 sulthonuladib sulthonuladib  88 Oct 14 20:42 bundles
-rw-r--r-- 1 sulthonuladib sulthonuladib  10 Oct 15 23:05 last_update_check
drwxr-xr-x 1 sulthonuladib sulthonuladib 426 Oct 14 20:44 skills
-rw-r--r-- 1 sulthonuladib sulthonuladib   1 Oct 14 20:42 tos_displayed
exit=0

$ cat ~/.android/cli/skills/version
1.0.16500706
exit=0

$ ls -la ~/.android/cli/skills
total 24
drwxr-xr-x 1 sulthonuladib sulthonuladib   426 Oct 14 20:44 .
drwxr-xr-x 1 sulthonuladib sulthonuladib   104 Oct 14 20:44 ..
drwxr-xr-x 1 sulthonuladib sulthonuladib    14 Oct 14 20:44 .agents
drwxr-xr-x 1 sulthonuladib sulthonuladib     6 Oct 14 20:44 build-system
drwxr-xr-x 1 sulthonuladib sulthonuladib    14 Oct 14 20:44 camera
drwxr-xr-x 1 sulthonuladib sulthonuladib    56 Oct 14 20:44 cars
drwxr-xr-x 1 sulthonuladib sulthonuladib    54 Oct 14 20:44 .claude-plugin
drwxr-xr-x 1 sulthonuladib sulthonuladib    22 Oct 14 20:44 .codex-plugin
-rw-r--r-- 1 sulthonuladib sulthonuladib    19 Oct 14 20:44 dac_skills.etag
drwxr-xr-x 1 sulthonuladib sulthonuladib    82 Oct 14 20:44 device-ai
drwxr-xr-x 1 sulthonuladib sulthonuladib    22 Oct 14 20:44 devtools
drwxr-xr-x 1 sulthonuladib sulthonuladib    46 Oct 14 20:44 .github
drwxr-xr-x 1 sulthonuladib sulthonuladib    66 Oct 14 20:44 identity
drwxr-xr-x 1 sulthonuladib sulthonuladib   102 Oct 14 20:44 jetpack-compose
-rw-r--r-- 1 sulthonuladib sulthonuladib 11357 Oct 14 20:44 LICENSE.txt
drwxr-xr-x 1 sulthonuladib sulthonuladib    46 Oct 14 20:44 media
drwxr-xr-x 1 sulthonuladib sulthonuladib    56 Oct 14 20:44 navigation
drwxr-xr-x 1 sulthonuladib sulthonuladib    22 Oct 14 20:44 performance
drwxr-xr-x 1 sulthonuladib sulthonuladib   156 Oct 14 20:44 play
drwxr-xr-x 1 sulthonuladib sulthonuladib    32 Oct 14 20:44 profilers
-rw-r--r-- 1 sulthonuladib sulthonuladib  2994 Oct 14 20:44 README.md
drwxr-xr-x 1 sulthonuladib sulthonuladib   102 Oct 14 20:44 security
drwxr-xr-x 1 sulthonuladib sulthonuladib    72 Oct 14 20:44 system
drwxr-xr-x 1 sulthonuladib sulthonuladib    26 Oct 14 20:44 testing
drwxr-xr-x 1 sulthonuladib sulthonuladib    64 Oct 14 20:44 tv
-rw-r--r-- 1 sulthonuladib sulthonuladib    12 Oct 14 20:44 version
drwxr-xr-x 1 sulthonuladib sulthonuladib    54 Oct 14 20:44 wear
drwxr-xr-x 1 sulthonuladib sulthonuladib    88 Oct 14 20:44 xr
exit=0
```

Bundled CLI runtime (not the project JDK): `~/.android/cli/bundles/7fef4dcca7bb46abf2b7ed3c99fa94f3f7369afd/`
contains a private `jre/` used to run the CLI itself, plus `main.jar`.

---

## B2 — `adb`

Captured 2026-10-15 ~23:06 +07 on `falcon`.

```
$ pgrep -a adb || echo none
51405 adb -L tcp:5037 fork-server server --reply-fd 4
exit=0            # pgrep matched, rc=0

$ adb version
Android Debug Bridge version 1.0.41
Version 37.0.0-android-tools
Installed as /usr/bin/adb
Running on Linux 7.2.5-3-omarchy (x86_64)
exit=0

$ adb devices -l
List of devices attached
exit=0

$ pgrep -a adb || echo none
51405 adb -L tcp:5037 fork-server server --reply-fd 4
exit=0            # pgrep matched, rc=0
```

**adb server side effect:** none from this audit. The server (PID 51405) was already running
*before* the first probe:

```
$ ps -o pid,lstart,cmd -p 51405
    PID                  STARTED CMD
  51405 Wed Oct 14 18:27:37 2026 adb -L tcp:5037 fork-server server --reply-fd 4
exit=0

$ cat ~/.android/adb.5037
/usr/bin/adb
exit=0
```

(`~/.android/adb.5037` mtime: `2026-10-14 18:27:38 +0700`.)

The same PID existed before and after `adb version` / `adb devices -l`; the audit did not
start, stop, or reconfigure it. No physical devices or emulators are attached
(`List of devices attached` is empty). `adb` binary is distro-provided
(`-rwxr-xr-x root root /usr/bin/adb`, 2026-09-05), not from a local SDK `platform-tools/`.

---

## B3 — Android SDK

Captured 2026-10-15 ~23:06–23:07 +07 on `falcon`.

```
$ echo "ANDROID_HOME=[$ANDROID_HOME] ANDROID_SDK_ROOT=[$ANDROID_SDK_ROOT] XDG_DATA_HOME=[$XDG_DATA_HOME]"
ANDROID_HOME=[] ANDROID_SDK_ROOT=[] XDG_DATA_HOME=[/home/sulthonuladib/.local/share]
exit=0

$ ls -la /home/sulthonuladib/Android/Sdk
ls: cannot access '/home/sulthonuladib/Android/Sdk': No such file or directory
exit=2

$ for d in "$HOME/Android/Sdk" /opt/android-sdk "$XDG_DATA_HOME/Android" /opt/android-sdk-linux "$HOME/android-sdk"; do ls -d "$d"; done
ls: cannot access '/home/sulthonuladib/Android/Sdk': No such file or directory
ls: cannot access '/opt/android-sdk': No such file or directory
ls: cannot access '/home/sulthonuladib/.local/share/Android': No such file or directory
ls: cannot access '/opt/android-sdk-linux': No such file or directory
ls: cannot access '/home/sulthonuladib/android-sdk': No such file or directory
exit=2

$ command -v sdkmanager || echo "sdkmanager: not on PATH"
sdkmanager: not on PATH
exit=0            # chain; command -v rc=1, fallback echo rc=0
$ command -v avdmanager || echo "avdmanager: not on PATH"
avdmanager: not on PATH
exit=0            # chain; command -v rc=1, fallback echo rc=0
$ command -v emulator || echo "emulator: not on PATH"
emulator: not on PATH
exit=0            # chain; command -v rc=1, fallback echo rc=0

$ find /home/sulthonuladib/Android/Sdk -maxdepth 3 \( -name sdkmanager -o -name avdmanager \) -print
(no output)
exit=1

$ ls -la /home/sulthonuladib/Android/Sdk/cmdline-tools
ls: cannot access '/home/sulthonuladib/Android/Sdk/cmdline-tools': No such file or directory
exit=2

$ timeout 30 find /home/sulthonuladib /opt /usr/local -maxdepth 6 -name 'sdkmanager*' -print | head -20
find: ‘/home/sulthonuladib/.docker-data/postgresql’: Permission denied
find: ‘/opt/containerd’: Permission denied
exit=0            # pipeline (head); PIPESTATUS: find=1 head=0; no stdout matches

$ for d in /home/sulthonuladib/Android /usr/lib/android-sdk /usr/share/android-sdk /home/sulthonuladib/.local/share/android; do ls -la "$d"; done
ls: cannot access '/home/sulthonuladib/Android': No such file or directory
ls: cannot access '/usr/lib/android-sdk': No such file or directory
ls: cannot access '/usr/share/android-sdk': No such file or directory
ls: cannot access '/home/sulthonuladib/.local/share/android': No such file or directory
exit=2
```

**Result:** no Android SDK is installed on this host at the CLI-reported path (`android info`)
or in any common location searched. Consequently there is no `cmdline-tools/`,
`sdkmanager`, `avdmanager`, `platforms/`, `build-tools/`, `platform-tools/`,
`system-images/`, `emulator/`, or `licenses/` content to list. `sdkmanager --list_installed` /
`--list` could not be used because `sdkmanager` is absent.

`android sdk list` (the CLI-native equivalent) was **not run**: it can fetch/update package
repository metadata, which the read-only constraint for this audit prohibits. No installed
SDK packages can therefore be enumerated; the accurate result is "SDK absent".

---

## B4 — Emulator / AVDs

Captured 2026-10-15 ~23:07 +07 on `falcon`.

```
$ command -v emulator || echo "emulator: not on PATH"
emulator: not on PATH
exit=0            # chain; command -v rc=1, fallback echo rc=0

$ ls -la ~/.android/avd
ls: cannot access '/home/sulthonuladib/.android/avd': No such file or directory
exit=2

$ timeout 60 android emulator list
(no output)
exit=0
```

`avdmanager` is not present (B3), and the SDK `emulator/` package is not installed (B3).
There are no AVD definitions to read (`~/.android/avd` absent), so no `config.ini` exists.
`android emulator list` reported no AVDs with exit 0 — only `list` was used; no
create/start/stop/remove was attempted.

---

## B5 — KVM / hardware acceleration

Captured 2026-10-15 ~23:07 +07 on `falcon`.

```
$ ls -l /dev/kvm
crw-rw-rw- 1 root kvm 10, 232 Oct 14 17:07 /dev/kvm
exit=0

$ test -r /dev/kvm && echo "readable" || echo "NOT readable"
readable
exit=0
$ test -w /dev/kvm && echo "writable" || echo "NOT writable"
writable
exit=0

$ id
uid=1000(sulthonuladib) gid=1000(sulthonuladib) groups=1000(sulthonuladib),966(docker),998(wheel)
exit=0

$ id | grep -o kvm || echo "kvm group: no"
kvm group: no
exit=0            # chain; grep rc=1, fallback echo rc=0

$ grep -m1 -oE 'vmx|svm' /proc/cpuinfo || echo "no vmx/svm found"
vmx
exit=0

$ lsmod | grep kvm
kvm_intel             532480  0
kvm                  1511424  1 kvm_intel
irqbypass              16384  1 kvm
exit=0
```

**Result:** `/dev/kvm` exists and is world-readable/-writable (`crw-rw-rw-`), so the current
user has read/write access even though it is not in the `kvm` group. CPU has `vmx`, and the
`kvm`/`kvm_intel` modules are loaded with `kvm_intel` in use. Hardware acceleration for an
x86_64 emulator is feasible once the SDK emulator and a system image are installed.

---

## B6 — JDK / mise selection

Captured 2026-10-15 ~23:07 +07 on `falcon`.

```
$ java -version
openjdk version "25.0.4.1" 2026-08-18 LTS
OpenJDK Runtime Environment Temurin-25.0.4.1+1 (build 25.0.4.1+1-LTS)
OpenJDK 64-Bit Server VM Temurin-25.0.4.1+1 (build 25.0.4.1+1-LTS, mixed mode, sharing)
exit=0

$ echo "JAVA_HOME=[$JAVA_HOME]"
JAVA_HOME=[/home/sulthonuladib/.local/share/mise/installs/java/temurin-25.0.4+101.0.LTS]
exit=0

$ command -v java
/home/sulthonuladib/.local/share/mise/installs/java/temurin-25/bin/java
exit=0

$ readlink -f "$(command -v java)"
/home/sulthonuladib/.local/share/mise/installs/java/temurin-25.0.4+101.0.LTS/bin/java
exit=0

$ readlink -f "$JAVA_HOME"
/home/sulthonuladib/.local/share/mise/installs/java/temurin-25.0.4+101.0.LTS
exit=0

$ mise -V
2026.10.5 linux-x64 (2026-10-08)
mise WARN  mise version 2026.10.10 available
mise WARN  self-update is disabled for this install, update mise the same way you installed it
exit=0

$ mise ls java
java  temurin-25.0.4+101.0.LTS  ~/.config/mise/config.toml  temurin-25
exit=0

$ mise current java
temurin-25.0.4+101.0.LTS
exit=0

$ mise which java
/home/sulthonuladib/.local/share/mise/installs/java/temurin-25/bin/java
exit=0
```

**How Temurin 25 is selected:** by the user-global mise config
`~/.config/mise/config.toml`:

```
$ cat ~/.config/mise/config.toml
[tools]
codex = "latest"
go = "latest"
java = "temurin-25"
node = "26.8.2"

[settings.upgrade]
auto_prune = false
exit=0
```

There is **no repo-level `mise.toml`** in this worktree (Phase 0), so the global pin applies.
`~/.local/share/mise/installs/java/` contains `temurin-25.0.4+101.0.LTS` and symlinks
`temurin-25`, `temurin-25.0`, `temurin-latest` → it. `/usr/lib/jvm` does not exist; the JDK is
mise-managed, and `JAVA_HOME` is exported by the mise shell hook as the versioned path.
No mutating mise commands were run.

---

## B7 — Gradle

Captured 2026-10-15 ~23:07 +07 on `falcon`.

```
$ command -v gradle || echo "gradle: not on PATH"
gradle: not on PATH
exit=1 (from command -v; the outer echo ran)

$ mise ls gradle
(no output)
exit=0

$ ls -la gradlew gradlew.bat gradle/wrapper
ls: cannot access 'gradlew': No such file or directory
ls: cannot access 'gradlew.bat': No such file or directory
ls: cannot access 'gradle/wrapper': No such file or directory
exit=2

$ find . -maxdepth 4 -name 'gradle-wrapper*' -not -path './.git/*'
(no output)
exit=0

$ ls -la apps
ls: cannot access 'apps': No such file or directory
exit=2
```

**Result (matches `AGENTS.md`):** system Gradle is absent, no mise Gradle is installed, and
the Phase 0 repository has no `gradlew`, no `gradle/wrapper/`, and no `apps/android`
directory.

**Plan only — from `full.md` §10.1 item 7 (not executed here):** "Commit the Gradle wrapper
and pin compatible plugins/dependencies; don't depend on a system Gradle version." The
Android project must therefore be scaffolded with a committed Gradle wrapper
(`gradlew` + `gradle/wrapper/gradle-wrapper.properties` + wrapper JAR) when `apps/android` is
created; the first wrapper run will download the configured Gradle distribution. The Gradle
and AGP version pins remain an open decision (see B8 and Gaps). No Gradle was installed and
no wrapper was generated by this audit.

---

## B8 — AGP / Gradle / JDK compatibility check (official docs, plain HTTP GET)

Retrieved 2026-10-15 ~23:08 +07 on `falcon` with `curl` (no authentication, no writes):

```
$ curl -sSL -o /dev/null -w 'final_url=%{url_effective}\nhttp_code=%{http_code}\nsize=%{size_download}\n' \
    --max-time 30 -A 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120 Safari/537.36' \
    https://developer.android.com/build/releases/gradle-plugin
final_url=https://developer.android.com/build/releases/agp-9-4-0-release-notes
http_code=200
size=281800
exit=0
```

AGP release notes, raw excerpt of the compatibility table (tags stripped only), retrieved
2026-10-15; page footer states "Last updated 2026-10-08 UTC":

```
Compatibility
The maximum API level that Android Gradle plugin 9.4 supports is API level 37.
Here is other compatibility info:

Minimum version   Default version   Notes
Gradle 9.6.0      9.6.0             To learn more, see updating Gradle.
SDK Build Tools 36.0.0  36.0.0       Install or configure SDK Build Tools.
NDK N/A           28.2.13676358     Install or configure a different version of the NDK.
JDK 17            17                To learn more, see setting the JDK version.
```

Gradle compatibility page (Gradle docs "current"):

```
$ curl -sSL -o /dev/null -w 'final_url=%{url_effective}\nhttp_code=%{http_code}\nsize=%{size_download}\n' \
    --max-time 30 -A 'Mozilla/5.0' https://docs.gradle.org/current/userguide/compatibility.html
final_url=https://docs.gradle.org/current/userguide/compatibility.html
http_code=200
size=178863
exit=0
```

Raw excerpt of "Table 1. Java Compatibility" (rows relevant to this host's JDK):

```
Java version  Support for toolchains  Support for running Gradle
17            7.3                     7.3 and after
18            7.5                     7.5 and after
19            7.6                     7.6 and after
20            8.1                     8.3 and after
21            8.4                     8.5 and after
22            8.7                     8.8 and after
23            8.10                    8.10 and after
24            8.14                    8.14 and after
25            9.1.0                   9.1.0 and after
26            9.4.0                   9.4.0 and after
27            9.8.0                   9.8.0 and after
```

**Assessment (docs-based, not yet tested):** Gradle can *run* on Java 25 only with Gradle
9.1.0 or later. The current AGP release notes (AGP 9.4) list a minimum Gradle of 9.6.0 and a
minimum/default JDK of 17. Combined, a Gradle wrapper ≥ 9.6.0 would satisfy both Gradle's
Java-25 runtime requirement and AGP 9.4's minimum Gradle. Whether AGP 9.4 is *tested/supported
on a JDK 25 runtime* is **not stated in the retrieved excerpts → UNVERIFIED**; this must be
confirmed by an actual wrapper build (Phase 5) or a project JDK pin decision. No claim of
compatibility is made here.

---

## B9 — Android Studio (informational only)

Captured 2026-10-15 ~23:07 +07 on `falcon`.

```
$ command -v studio || echo "studio: not on PATH"
studio: not on PATH
exit=0            # chain; command -v rc=1, fallback echo rc=0

$ for d in /opt/android-studio /usr/local/android-studio "$HOME/android-studio" \
           "$HOME/.local/share/JetBrains" /opt/google/android-studio /snap/android-studio; do ls -d "$d"; done
ls: cannot access '/opt/android-studio': No such file or directory
ls: cannot access '/usr/local/android-studio': No such file or directory
ls: cannot access '/home/sulthonuladib/android-studio': No such file or directory
ls: cannot access '/home/sulthonuladib/.local/share/JetBrains': No such file or directory
ls: cannot access '/opt/google/android-studio': No such file or directory
ls: cannot access '/snap/android-studio': No such file or directory
exit=2

$ for c in adb studio emulator sdkmanager avdmanager gradle java; do printf '%-12s: ' "$c"; command -v "$c" || echo "not found"; done
adb         : /usr/bin/adb
studio      : not found
emulator    : not found
sdkmanager  : not found
avdmanager  : not found
gradle      : not found
java        : /home/sulthonuladib/.local/share/mise/installs/java/temurin-25/bin/java
exit=0            # chain; last iteration (java) found, command -v rc=0
```

**Result:** Android Studio is not installed in any common location and is not on `PATH`. Per
`full.md` §10, Android Studio is **not** required and must not become a build dependency; the
build/test workflow is Android CLI + Gradle wrapper + SDK command-line tools + adb/emulator.

---

## B10 — Gaps blocking implementation

| # | Gap | Impact | Safest next step (requires user decision where noted) |
|---|---|---|---|
| G1 | **No Android SDK installed.** `android info` reports `/home/sulthonuladib/Android/Sdk`, but that path, and all common alternatives, are absent; `ANDROID_HOME`/`ANDROID_SDK_ROOT` unset. | Blocks `sdkmanager`/`avdmanager`, `platforms`, `build-tools`, `platform-tools` from SDK, and any Gradle Android build. | User approval to install an SDK (location + method), e.g. via `android sdk install` or SDK command-line tools. ~GB download; must be an explicit decision per `full.md` §0.3.10. |
| G2 | **`sdkmanager`/`avdmanager` absent** (part of G1). | Cannot enumerate/install packages or manage AVDs with the canonical CLI tools. | Follows from G1; verify `cmdline-tools` install and then re-run `--list`. |
| G3 | **No emulator binary, no system images, no AVDs** (`emulator` not on PATH, `~/.android/avd` absent, `android emulator list` empty). | Blocks emulator-based runs/Journeys (`full.md` §10.4). | After G1: pick an API level/system image (`android emulator create` / `sdkmanager`), which requires a large download → user approval. Hardware acceleration is available (B5). |
| G4 | **Gradle absent and no wrapper in repo** (expected for Phase 0). | No Android build can run until `apps/android` is scaffolded with a committed wrapper. | In Phase 5, scaffold with `android create` and commit `gradlew` + wrapper; first run downloads the pinned distribution. Do not install system Gradle (`AGENTS.md`). |
| G5 | **Gradle/AGP version pins undecided.** AGP 9.4 (current docs) needs Gradle ≥ 9.6.0, SDK Build Tools 36.0.0, JDK ≥ 17; Gradle runs on Java 25 only from 9.1.0. AGP 9.4-on-JDK-25 support is not documented in the retrieved excerpts. | Wrong pin → non-reproducible or failing Android builds. | Decide `gradle-wrapper.properties` + AGP version at Phase 5 scaffolding; validate with a real `./gradlew` build before claiming support. Record as an ADR if the choice is non-obvious. |
| G6 | **Only JDK 25 (Temurin) is installed**, via user-global mise pin; no repo `mise.toml`. | AGP docs state a *minimum* JDK 17 and do not state a JDK 25 maximum; however AGP is commonly validated on current LTS JDKs. A JDK 25-only host is a compatibility risk for AGP/Gradle. | Proposal only (not executed): consider a project-scoped `mise.toml` or wrapper toolchain pin (e.g. Temurin 21 or 25) once a real build is attempted. Requires user approval; do not silently change global Java. |
| G7 | **SDK licenses not accepted** (no `licenses/` dir; nothing installed). | Installing packages requires license acceptance, a user-authorized action. | Handle during G1 installation by the user/documented procedure; this audit intentionally took no license action. |
| G8 | **No device or emulator attached** (`adb devices -l` empty). | No on-device verification possible yet. | Expected at Phase 0; revisit after G1/G3. |
| G9 | **`apps/android` does not exist**; repo has no Gradle files (`full.md` §5.1 sketch not yet built). | Android work cannot start until scaffolding. | AND-001/Phase 5 task after SDK + JDK/Gradle decisions. |
| G10 | **`ANDROID_HOME` unset and `local.properties` absent.** | Gradle/AGP will need the SDK location once the project exists. | Plan: set `sdk.dir` in a gitignored `local.properties` or export `ANDROID_HOME`; decide and document at scaffold time. Do not hardcode. |
| G11 | **Android CLI analytics/update side files touched** (`last_update_check`, `analytics/last_upload_time` updated during this audit; see B11). Analytics opt-out not inspected beyond file presence. | Minor privacy/noise consideration for `full.md` §10.1.9. | Read `android skills`/CLI docs for a documented opt-out when Android work actually starts; do not change config without approval. |

Not a gap: **KVM acceleration is available** (`/dev/kvm` accessible, `vmx`, `kvm_intel`
loaded), and **`adb` works** (server already running, no devices attached).

---

## B11 — What changed on this host

- **No installations, downloads, builds, license acceptances, service/port changes, or
  `sudo` operations were performed.** All probes were read-only except where a tool updated
  its own bookkeeping file.
- **adb:** no change. An adb server (PID 51405, started 2026-10-14 18:27:37) was already
  running before the first probe and was the same PID afterwards. No device state changed.
- **Android CLI bookkeeping:** the CLI updated its own timestamp files while being invoked
  during this audit:
  - `~/.android/cli/last_update_check` → `1792080315` (`2026-10-15T23:05:15+07:00`),
    mtime `2026-10-15 23:05:15 +0700`;
  - `~/.android/cli/analytics/last_upload_time` → same epoch/mtime.
  These are Android CLI-managed files, touched by running `android -V`/`android info`/help
  probes; this audit did not write them directly. The `android init` state
  (`~/.android/cli/skills`, `bundles`) predates the audit (2026-10-14 20:41–20:44) and was
  not modified.
- **Repository:** `git status --short` was empty before the probes. The only file written by
  this audit is `docs/operations/android-toolchain.md`; `docs/operations/toolchain-audit.md`
  belongs to the parallel ENV-001 worker and was not modified (it already existed when this
  audit began writing).
- **SDK/emulator/AVD/Gradle/Java:** no state existed to change; nothing was installed or
  configured.

---

## Sources

- Host probes: this document (commands and raw output above; cwd/branch/HEAD in the header).
- `full.md` §0.4, §10 (Android CLI workflow), §13.5 (build matrix) — task acceptance context.
- `AGENTS.md` — verified-local-environment summary and non-negotiables.
- AGP 9.4 release notes compatibility table:
  https://developer.android.com/build/releases/agp-9-4-0-release-notes (retrieved 2026-10-15;
  page "Last updated 2026-10-08 UTC"; arrived via redirect from
  `.../build/releases/gradle-plugin`).
- Gradle "Compatibility" page, Table 1 (Java Compatibility):
  https://docs.gradle.org/current/userguide/compatibility.html (retrieved 2026-10-15).
