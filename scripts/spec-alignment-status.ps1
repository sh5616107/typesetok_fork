#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Check status of spec alignment project

.DESCRIPTION
    Shows progress of all 5 phases, test results, and key metrics

.EXAMPLE
    .\scripts\spec-alignment-status.ps1
#>

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent

Write-Host "=== TypesetOK Spec Alignment Status ===" -ForegroundColor Cyan
Write-Host ""

# Check if we're on feature branch
$currentBranch = git -C $root rev-parse --abbrev-ref HEAD
Write-Host "Current Branch: " -NoNewline
if ($currentBranch -like "feature/spec-alignment*") {
    Write-Host $currentBranch -ForegroundColor Green
} else {
    Write-Host $currentBranch -ForegroundColor Yellow
    Write-Host "  (Not on spec-alignment branch)" -ForegroundColor Yellow
}
Write-Host ""

# Phase 1: Check for new files
Write-Host "Phase 1: Infrastructure" -ForegroundColor Cyan
$phase1Files = @(
    "crates/tok-typeset/src/mpt.rs",
    "crates/tok-typeset/src/layout_result.rs",
    "crates/tok-typeset/src/diagnostic.rs",
    "crates/tok-typeset/src/page_policy.rs",
    "crates/tok-typeset/src/waiver.rs"
)

foreach ($file in $phase1Files) {
    $path = Join-Path $root $file
    if (Test-Path $path) {
        Write-Host "  ✓ $file" -ForegroundColor Green
    } else {
        Write-Host "  ✗ $file" -ForegroundColor Gray
    }
}
Write-Host ""

# Phase 2: Check for measurement
Write-Host "Phase 2: Measurement & Policy" -ForegroundColor Cyan
$phase2Files = @(
    "crates/tok-typeset/src/balance_measurement.rs",
    "tests/spec_balance_basic.rs"
)

foreach ($file in $phase2Files) {
    $path = Join-Path $root $file
    if (Test-Path $path) {
        Write-Host "  ✓ $file" -ForegroundColor Green
    } else {
        Write-Host "  ✗ $file" -ForegroundColor Gray
    }
}
Write-Host ""

# Phase 3: Check for solver
Write-Host "Phase 3: Balance Solver" -ForegroundColor Cyan
$phase3Files = @(
    "crates/tok-typeset/src/balance_solver.rs",
    "crates/tok-typeset/src/candidate.rs",
    "crates/tok-typeset/src/scoring.rs"
)

foreach ($file in $phase3Files) {
    $path = Join-Path $root $file
    if (Test-Path $path) {
        Write-Host "  ✓ $file" -ForegroundColor Green
    } else {
        Write-Host "  ✗ $file" -ForegroundColor Gray
    }
}
Write-Host ""

# Phase 4: Check for continuous flow
Write-Host "Phase 4: Continuous Flow" -ForegroundColor Cyan
$phase4Files = @(
    "crates/tok-typeset/src/snake_column_breaker.rs"
)

foreach ($file in $phase4Files) {
    $path = Join-Path $root $file
    if (Test-Path $path) {
        Write-Host "  ✓ $file" -ForegroundColor Green
    } else {
        Write-Host "  ✗ $file" -ForegroundColor Gray
    }
}
Write-Host ""

# Test Results
Write-Host "Test Results" -ForegroundColor Cyan
Push-Location $root
try {
    Write-Host "  Running cargo test..." -ForegroundColor Gray
    $testOutput = cargo test --quiet 2>&1 | Out-String
    
    if ($testOutput -match "(\d+) passed") {
        $passed = $matches[1]
        Write-Host "  ✓ $passed tests passing" -ForegroundColor Green
    }
    
    if ($testOutput -match "(\d+) failed") {
        $failed = $matches[1]
        Write-Host "  ✗ $failed tests failing" -ForegroundColor Red
    }
} catch {
    Write-Host "  Error running tests: $_" -ForegroundColor Red
} finally {
    Pop-Location
}
Write-Host ""

# Spec Alignment Tests
Write-Host "Spec Alignment Tests (Section 16)" -ForegroundColor Cyan
$specTests = @(
    "spec_section_16_1_balance",
    "spec_section_16_2_typography", 
    "spec_section_16_3_determinism"
)

foreach ($testSuite in $specTests) {
    $testFile = "tests/$testSuite.rs"
    $path = Join-Path $root $testFile
    
    if (Test-Path $path) {
        Push-Location $root
        try {
            $result = cargo test --test $testSuite --quiet 2>&1 | Out-String
            if ($result -match "(\d+) passed") {
                Write-Host "  ✓ $testSuite: $($matches[1]) passed" -ForegroundColor Green
            } else {
                Write-Host "  ? $testSuite: present" -ForegroundColor Yellow
            }
        } catch {
            Write-Host "  ? $testSuite: error" -ForegroundColor Yellow
        } finally {
            Pop-Location
        }
    } else {
        Write-Host "  ✗ $testSuite: not yet implemented" -ForegroundColor Gray
    }
}
Write-Host ""

# Documentation Status
Write-Host "Documentation" -ForegroundColor Cyan
$docs = @(
    "SPEC_ALIGNMENT.md",
    "docs/PHASE2_PLAN.md",
    "docs/PHASE3_PLAN.md",
    "docs/PHASE4_PLAN.md",
    "docs/PHASE5_PLAN.md"
)

foreach ($doc in $docs) {
    $path = Join-Path $root $doc
    if (Test-Path $path) {
        $size = (Get-Item $path).Length
        Write-Host "  ✓ $doc ($([math]::Round($size/1KB, 1)) KB)" -ForegroundColor Green
    } else {
        Write-Host "  ✗ $doc" -ForegroundColor Gray
    }
}
Write-Host ""

# Quick Stats
Write-Host "Quick Stats" -ForegroundColor Cyan
Push-Location $root
try {
    $rustFiles = (Get-ChildItem -Path "crates/tok-typeset/src" -Filter "*.rs" -Recurse).Count
    Write-Host "  Rust files in tok-typeset: $rustFiles" -ForegroundColor Gray
    
    $testFiles = (Get-ChildItem -Path "tests" -Filter "*.rs" -ErrorAction SilentlyContinue).Count
    Write-Host "  Test files: $testFiles" -ForegroundColor Gray
    
    $gitStatus = git status --short
    if ($gitStatus) {
        $changes = ($gitStatus | Measure-Object).Count
        Write-Host "  Uncommitted changes: $changes files" -ForegroundColor Yellow
    } else {
        Write-Host "  Uncommitted changes: 0" -ForegroundColor Green
    }
} finally {
    Pop-Location
}
Write-Host ""

Write-Host "=== End of Status ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "For detailed tracking, see: SPEC_ALIGNMENT.md" -ForegroundColor Gray
