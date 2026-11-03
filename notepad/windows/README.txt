Patrone Notepad for Windows
===========================

Plain text and Markdown, in tabs: PatroneOS's text editor, built for
64-bit Windows 10 and 11. It needs no administrator rights and changes no
file associations.

To install it for your account:

  1. Unpack this zip into any folder.
  2. In that folder, right-click install.ps1 and choose "Run with
     PowerShell" (or, in PowerShell there:
     powershell -ExecutionPolicy Bypass -File .\install.ps1).
  3. Open Patrone Notepad from the Start menu.

It goes in %LOCALAPPDATA%\Programs\Patrone Notepad, and keeps its settings
and open tabs in %LOCALAPPDATA%\patrone-notepad.

To remove it, run uninstall.ps1 the same way (add -RemoveSettings to
remove its settings and kept tabs too).

PatroneOS's fonts (SIL Open Font License 1.1) come with it, in
share\fonts\patroneos; it uses them itself and doesn't install them in
Windows. The GTK libraries it runs on come from MSYS2; their licences and
where their source is are in share\licenses.

Source: https://github.com/billsecond/patroneos (notepad/)
