$Env:RUSTFLAGS = "--cfg=web_sys_unstable_apis"
cat .\build_dev.sh | Invoke-Expression
