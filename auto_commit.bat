@echo off
setlocal
echo =========================================
echo       GIT AUTO-COMMIT E PUSH
echo =========================================
echo.

:: Pega a versao do projeto automaticamente lendo o Cargo.toml
set VERSION=latest
for /f "tokens=3 delims= " %%a in ('findstr /R "^version" Cargo.toml 2^>nul') do (
    set VERSION=%%~a
    goto :found_version
)
:found_version

:: Monta a mensagem final automatizada
set msg=Automated update (v%VERSION%)

echo [1/4] Adicionando arquivos ao Git...
git add .

echo [2/4] Criando o commit com a mensagem: "%msg%"
git commit -m "%msg%"

echo [3/4] Sincronizando repositorio (Pull)...
git pull --rebase

echo [4/4] Enviando para o GitHub (Push)...
git push

echo.
echo =========================================
echo        PROCESSO CONCLUIDO!
echo =========================================
pause
