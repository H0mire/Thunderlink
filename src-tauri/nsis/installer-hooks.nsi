; Thunderlink NSIS Installer Hooks
; Adds firewall rules automatically during installation

!macro NSIS_HOOK_POSTINSTALL
  ; Add firewall rule for the application (allows all traffic)
  DetailPrint "Adding Windows Firewall rules for Thunderlink..."
  
  ; Allow Thunderlink.exe through firewall (inbound)
  nsExec::ExecToLog 'netsh advfirewall firewall add rule name="Thunderlink" dir=in action=allow program="$INSTDIR\Thunderlink.exe" enable=yes profile=private,public'
  
  ; Allow UDP port 42069 for peer discovery
  nsExec::ExecToLog 'netsh advfirewall firewall add rule name="Thunderlink Discovery (UDP)" dir=in action=allow protocol=UDP localport=42069 enable=yes profile=private,public'
  
  ; Allow TCP port 5354 for file transfer
  nsExec::ExecToLog 'netsh advfirewall firewall add rule name="Thunderlink Transfer (TCP)" dir=in action=allow protocol=TCP localport=5354 enable=yes profile=private,public'
  
  DetailPrint "Firewall rules added successfully."
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Remove firewall rules when uninstalling
  DetailPrint "Removing Windows Firewall rules for Thunderlink..."
  
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="Thunderlink"'
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="Thunderlink Discovery (UDP)"'
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="Thunderlink Transfer (TCP)"'
  
  DetailPrint "Firewall rules removed."
!macroend

