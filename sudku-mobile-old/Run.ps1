#[System.Environment]::SetEnvironmentVariable("JAVA_HOME", "C:\Program Files\Android\Android Studio\jbr", "User")

#[System.Environment]::SetEnvironmentVariable("ANDROID_HOME", "$env:LocalAppData\Android\Sdk", "User")
# TODO: The version varies, fix this
#[System.Environment]::SetEnvironmentVariable("NDK_HOME", "$env:LocalAppData\Android\Sdk\ndk\26.1.10909125", "User")

$env:JAVA_HOME = "C:\Program Files\Android\Android Studio\jbr"

$env:ANDROID_HOME = "$env:LocalAppData\Android\Sdk"
# TODO: The version varies, fix this
$env:NDK_HOME = "$env:LocalAppData\Android\Sdk\ndk\26.1.10909125"

cargo tauri android dev
