param([Parameter(Mandatory)][int]$AppProcessId,[Parameter(Mandatory)][string]$SamplePath)
$ErrorActionPreference='Stop'
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class ReconXPicker {
 public delegate bool Callback(IntPtr h, IntPtr p);
 [DllImport("user32.dll")] public static extern bool EnumWindows(Callback c, IntPtr p);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr h, Callback c, IntPtr p);
 [DllImport("user32.dll")] public static extern int GetDlgCtrlID(IntPtr h);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr SendMessage(IntPtr h, uint m, IntPtr w, string text);
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, System.Text.StringBuilder b, int n);
 public static string Describe(IntPtr h) {
  var title=new System.Text.StringBuilder(256);GetWindowText(h,title,256);
  var result="Dialog title: "+title.ToString()+"; controls: ";
  EnumChildWindows(h,(c,p)=>{var b=new System.Text.StringBuilder(256);GetClassName(c,b,256);result+=b.ToString()+"#"+GetDlgCtrlID(c)+" ";return true;},IntPtr.Zero);return result;
 }
 public static IntPtr Control(IntPtr dialog,string kind,int id) {
  IntPtr found=IntPtr.Zero;EnumChildWindows(dialog,(h,p)=>{var b=new System.Text.StringBuilder(256);GetClassName(h,b,256);if(b.ToString()==kind && GetDlgCtrlID(h)==id){found=h;return false;}return true;},IntPtr.Zero);return found;
 }
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h, System.Text.StringBuilder b, int n);
 public static IntPtr Find(int pid) {
  IntPtr found=IntPtr.Zero;
  EnumWindows((h,p)=>{uint owner;GetWindowThreadProcessId(h,out owner);var b=new System.Text.StringBuilder(256);GetClassName(h,b,256);if(owner==pid && IsWindowVisible(h) && b.ToString()=="#32770"){found=h;return false;}return true;},IntPtr.Zero);
  return found;
 }
}
'@
$pickerHandle=[IntPtr]::Zero
for($attempt=0;$attempt -lt 80;$attempt++) {
 $pickerHandle=[ReconXPicker]::Find($AppProcessId)
 if($pickerHandle -ne [IntPtr]::Zero){break}
 Start-Sleep -Milliseconds 200
}
if($pickerHandle -eq [IntPtr]::Zero){throw 'No native file dialog found in the owned test application'}
$filename=[IntPtr]::Zero
for($attempt=0;$attempt -lt 80;$attempt++) {
 $filename=[ReconXPicker]::Control($pickerHandle,'Edit',1148)
 if($filename -eq [IntPtr]::Zero){$filename=[ReconXPicker]::Control($pickerHandle,'Edit',1001)}
 if($filename -ne [IntPtr]::Zero){break}
 Start-Sleep -Milliseconds 100
}
if($filename -eq [IntPtr]::Zero){Write-Output ([ReconXPicker]::Describe($pickerHandle));throw 'Native filename input was not found'}
[void][ReconXPicker]::SendMessage($filename,0x000C,[IntPtr]::Zero,$SamplePath)
$openButton=[ReconXPicker]::Control($pickerHandle,'Button',1)
if($openButton -eq [IntPtr]::Zero){throw 'Native Open button was not found'}
if(-not [ReconXPicker]::PostMessage($openButton,0x00F5,[IntPtr]::Zero,[IntPtr]::Zero)){throw 'Native Open action failed'}
Write-Output 'Selected an owned sample through the native file dialog'
