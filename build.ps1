param(
    [Parameter(Position=0)]
    [string]$Action = "build"
)

function Build-TypeScript {
    Set-Location typescript
    tsc
    if ($LASTEXITCODE -ne 0) { exit 1 }
    Set-Location ..
}

function Build-Rust {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { exit 1 }
}

function Build-Python {
    python -m build
    if ($LASTEXITCODE -ne 0) { exit 1 }
}

function Publish-TypeScript {
    npm publish
    if ($LASTEXITCODE -ne 0) { exit 1 }
}

function Publish-Rust {
    cargo publish --allow-dirty
    if ($LASTEXITCODE -ne 0) { exit 1 }
}

function Publish-Python {
    twine upload dist/*
    if ($LASTEXITCODE -ne 0) { exit 1 }
}

function Clean-All {
    if (Test-Path "typescript\dist") { Remove-Item -Recurse -Force "typescript\dist" }
    cargo clean
    if (Test-Path "dist") { Remove-Item -Recurse -Force "dist" }
    if (Test-Path "build") { Remove-Item -Recurse -Force "build" }
    Get-ChildItem -Path . -Filter "*.egg-info" -Recurse | Remove-Item -Recurse -Force
}

switch ($Action) {
    "build" {
        Build-TypeScript
        Build-Rust
        Build-Python
    }
    "publish" {
        Publish-TypeScript
        Publish-Rust
        Publish-Python
    }
    "clean" {
        Clean-All
    }
    "release" {
        Build-TypeScript
        Build-Rust
        Build-Python
        Publish-TypeScript
        Publish-Rust
        Publish-Python
    }
    default {
        Write-Host "Usage: .\scripts\build.ps1 [build|publish|clean|release]"
        exit 1
    }
}
