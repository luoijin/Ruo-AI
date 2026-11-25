param(
    [string]$Branch
)

$ErrorActionPreference = 'Stop'
$repository = 'luoijin/Ruo-AI'
$artifactName = 'gate2-sqlcipher-arm64'
$workflowFile = 'ci.yml'
$artifactPath = 'android\core\rustbridge\src\main\jniLibs\arm64-v8a\libruo_core.so'
$repoRoot = Split-Path -Parent $PSScriptRoot

$ghCommand = Get-Command gh -ErrorAction SilentlyContinue
if ($ghCommand) {
    $gh = $ghCommand.Source
} else {
    $installedGh = Join-Path $env:ProgramFiles 'GitHub CLI\gh.exe'
    if (-not (Test-Path -LiteralPath $installedGh)) {
        throw 'GitHub CLI (gh) was not found on PATH or in its standard Windows install directory.'
    }
    $gh = $installedGh
}

if ([string]::IsNullOrWhiteSpace($Branch)) {
    $currentBranch = & git -C $repoRoot branch --show-current
    if ($LASTEXITCODE -ne 0) {
        throw 'Git could not determine the current branch; pass -Branch explicitly.'
    }
    $Branch = $currentBranch | Select-Object -First 1
    if ($null -ne $Branch) {
        $Branch = $Branch.Trim()
    }
    if ([string]::IsNullOrWhiteSpace($Branch)) {
        throw 'Pass -Branch when the current checkout is detached or Git cannot determine the branch.'
    }
}

$runOutput = & $gh run list `
    --repo $repository `
    --workflow $workflowFile `
    --branch $Branch `
    --status success `
    --limit 50 `
    --json databaseId,headSha,createdAt 2>&1
if ($LASTEXITCODE -ne 0) {
    throw 'gh could not list successful CI runs; authenticate with `gh auth login` and retry.'
}

$runs = ($runOutput -join [Environment]::NewLine) | ConvertFrom-Json
$run = $runs | Sort-Object { [DateTime]$_.createdAt } -Descending | Select-Object -First 1
if (-not $run) {
    throw "No successful '$workflowFile' run was found for branch '$Branch'."
}

$downloadDirectory = Join-Path ([System.IO.Path]::GetTempPath()) "ruo-gate2-$([Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $downloadDirectory | Out-Null

try {
    & $gh run download $run.databaseId `
        --repo $repository `
        --name $artifactName `
        --dir $downloadDirectory
    if ($LASTEXITCODE -ne 0) {
        throw "Could not download '$artifactName' from CI run $($run.databaseId)."
    }

    $libraries = @(Get-ChildItem -LiteralPath $downloadDirectory -Filter 'libruo_core.so' -File -Recurse)
    if ($libraries.Count -ne 1) {
        throw "Expected one libruo_core.so in the artifact; found $($libraries.Count)."
    }
    if ($libraries[0].Length -eq 0) {
        throw 'The downloaded libruo_core.so is empty.'
    }

    $destination = Join-Path $repoRoot $artifactPath
    $destinationDirectory = Split-Path -Parent $destination
    New-Item -ItemType Directory -Path $destinationDirectory -Force | Out-Null
    Copy-Item -LiteralPath $libraries[0].FullName -Destination $destination -Force

    Write-Output "Downloaded libruo_core.so from CI run $($run.databaseId) ($($run.headSha)) to $destination."
} finally {
    if (Test-Path -LiteralPath $downloadDirectory) {
        Remove-Item -LiteralPath $downloadDirectory -Recurse -Force
    }
}
