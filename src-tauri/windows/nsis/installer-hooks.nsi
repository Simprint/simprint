!include "FileFunc.nsh"

!insertmacro GetFileName

Var PromoCode
Var RuntimePathValue
Var RuntimePathRoot

; 从当前运行中的安装包文件名提取完整文件名（包含 .exe 扩展名），原样写入 referral 目录。
; 当前客户端注册逻辑只会识别文件名中的 `R_<base64url(referral_code)>_R` 标记。
; 示例：
;   simprint_setup-R_SU5WNzZGNDUwN0Y_R.exe  -> 可识别的邀请码载体
;   simprint_setup-1BL5PQP.exe              -> 不会被识别为邀请码
;   simprint-setup.exe                      -> 普通安装包，不包含邀请码

!macro NSIS_HOOK_POSTINSTALL
  SetShellVarContext current

  ; 1. 取得当前安装包文件名（NSIS 内置 $EXEFILE = 正在运行的安装程序 exe 完整路径）
  ; 2. 提取文件名部分（包含 .exe），使用完整文件名作为推广码文件名
  ${GetFileName} $EXEFILE $PromoCode

  ; 3. 先删除旧的 referral 目录，再创建目录并写入空文件
  RMDir /r "$LOCALAPPDATA\Simprint\referral"
  StrCmp $PromoCode "" NoPromoFile
  CreateDirectory "$LOCALAPPDATA\Simprint\referral"
  FileOpen $0 "$LOCALAPPDATA\Simprint\referral\$PromoCode" w
  FileClose $0

NoPromoFile:
!macroend

; 卸载前钩子：递归清理安装目录内所有文件与子目录（含运行时生成内容），卸载程序自身由后续步骤或重启后清理
!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "正在清理安装目录内所有文件..."
  Push $R0
  Push $R1
  Push $R2

  FindFirst $R0 $R1 "$INSTDIR\*.*"
loop_start:
  StrCmp $R1 "" loop_end
  StrCmp $R1 "." loop_next
  StrCmp $R1 ".." loop_next

  StrCpy $R2 "$INSTDIR\$R1"
  IfFileExists "$R2\*.*" 0 +3
    RMDir /r "$R2"
    Goto loop_next
  Delete "$R2"

loop_next:
  FindNext $R0 $R1
  Goto loop_start

loop_end:
  FindClose $R0

  Pop $R2
  Pop $R1
  Pop $R0

  DetailPrint "目录清理完成"
!macroend

Function un.RemoveRuntimePathIfSafe
  Exch $RuntimePathValue

  StrCmp $RuntimePathValue "" done
  StrCmp $RuntimePathValue "$LOCALAPPDATA" done
  StrCmp $RuntimePathValue "$APPDATA" done
  StrCmp $RuntimePathValue "$INSTDIR" done

  ${GetRoot} "$RuntimePathValue" $RuntimePathRoot
  StrCmp $RuntimePathValue "$RuntimePathRoot" done

  RmDir /r "$RuntimePathValue"

done:
FunctionEnd

!macro NSIS_HOOK_POSTUNINSTALL
  ; 仅在用户勾选“删除应用程序数据”且不是更新流程时执行
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    ; 清理历史遗留的 Tauri 默认目录
    SetShellVarContext current
    RmDir /r "$APPDATA\${BUNDLEID}"
    RmDir /r "$LOCALAPPDATA\${BUNDLEID}"

    ReadRegStr $RuntimePathValue HKCU "Software\${PRODUCTNAME}\RuntimePaths" "LogsDir"
    Push $RuntimePathValue
    Call un.RemoveRuntimePathIfSafe

    ReadRegStr $RuntimePathValue HKCU "Software\${PRODUCTNAME}\RuntimePaths" "CacheDir"
    Push $RuntimePathValue
    Call un.RemoveRuntimePathIfSafe

    ReadRegStr $RuntimePathValue HKCU "Software\${PRODUCTNAME}\RuntimePaths" "KernelsDir"
    Push $RuntimePathValue
    Call un.RemoveRuntimePathIfSafe

    ReadRegStr $RuntimePathValue HKCU "Software\${PRODUCTNAME}\RuntimePaths" "ReferralDir"
    Push $RuntimePathValue
    Call un.RemoveRuntimePathIfSafe

    ReadRegStr $RuntimePathValue HKCU "Software\${PRODUCTNAME}\RuntimePaths" "ConfigDir"
    Push $RuntimePathValue
    Call un.RemoveRuntimePathIfSafe

    ReadRegStr $RuntimePathValue HKCU "Software\${PRODUCTNAME}\RuntimePaths" "DataDir"
    Push $RuntimePathValue
    Call un.RemoveRuntimePathIfSafe

    ; 默认根目录始终用 NSIS 原生命令删除
    RmDir /r "$LOCALAPPDATA\Simprint"
  ${EndIf}

  DeleteRegKey HKCU "Software\${PRODUCTNAME}\RuntimePaths"
!macroend
