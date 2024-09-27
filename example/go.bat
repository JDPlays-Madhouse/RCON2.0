@echo off


:Start
rem Launch the bot.
echo Launching TwitchChattoRCON Bot...  (CTRL-C to stop bot)
node "bot-battlefluffy-scenario_MUPPET_2022-12-18"


rem This part will restart the batch file easily.
@set /p reconnect="Reconnect (Y/n): "
@if "%reconnect%"=="" set reconnect=Y

if "%reconnect%"=="Y" goto Start
if "%reconnect%"=="y" goto Start


rem Don't let the screen just blink out of existance.
@pause
