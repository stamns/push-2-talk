  # 方式 1：带 v 前缀
  # .\tag-release.ps1 v0.0.8

  # 方式 2：不带 v 前缀（会自动添加）
  # .\tag-release.ps1 0.0.8

param(
    [Parameter(Mandatory=$true, Position=0)]
    [string]$Version
)

# 确保版本号以 v 开头
if (-not $Version.StartsWith("v")) {
    $Version = "v$Version"
}

Write-Host "Creating tag: $Version" -ForegroundColor Cyan

# 创建 tag
git tag $Version
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to create tag" -ForegroundColor Red
    exit 1
}

Write-Host "Pushing main branch to github..." -ForegroundColor Cyan

# 推送 main 分支
git push github main
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to push main branch" -ForegroundColor Red
    exit 1
}

Write-Host "Pushing tag $Version to github..." -ForegroundColor Cyan

# 推送 tag
git push github $Version
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to push tag" -ForegroundColor Red
    exit 1
}

Write-Host "Done! Tag $Version has been created and pushed." -ForegroundColor Green
