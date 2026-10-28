; Bati installer settings, included by Tauri before the pages are declared.

; Bandeau Bati on the right of the header of the intermediate pages.
!define MUI_HEADERIMAGE_RIGHT

; Finish page: "Lancer Bati" (checked by default, from Tauri) and a "Terminer" button.
!define MUI_FINISHPAGE_BUTTON "Terminer"

; "Installation terminee" moves on by itself to the finish page.
!macro NSIS_HOOK_POSTINSTALL
  SetAutoClose true
!macroend
