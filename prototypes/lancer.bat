@echo off
rem Lance les trois prototypes d'interface (React 5171, Svelte 5172, Solid 5173) puis les ouvre.
cd /d "%~dp0"
for %%d in (react svelte solid) do (
  if not exist %%d\node_modules (pushd %%d & call npm install --no-audit --no-fund & popd)
  pushd %%d & call npx vite build & popd
)
start "React 5171" /min cmd /c "cd react && npx vite preview --port 5171 --strictPort"
start "Svelte 5172" /min cmd /c "cd svelte && npx vite preview --port 5172 --strictPort"
start "Solid 5173" /min cmd /c "cd solid && npx vite preview --port 5173 --strictPort"
timeout /t 4 >nul
start http://localhost:5171/
start http://localhost:5172/
start http://localhost:5173/
echo Fermez les trois fenetres reduites pour arreter les prototypes.
pause
