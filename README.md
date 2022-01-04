# TinyWeather
__Currently a WIP__

## Introduction
TinyWeather is a small personal weather monitoring system. The system is made of
two components: the controller process (this repo) and the [station](https://github.com/dcrussell/tw_station).
The image below provides a high-level overview of the architecture:
![system architecture](resources/TinyWeather.png)


# Controller
The purpose of the controller process is to remotely control the station, process the weather data,
and store it for later use. Exchange between the controller and the station is done using serial
over either bluetooth or USB. All settings, such as the baud rate and device path
are provided in a configuration file.


## Setup

### Config file
The configuration file is a simple text file using a hierachical dot notation
syntax. Example:

```
# This is a comment
serial.baud=9600 # This provides the baud rate for the serial communication

log.file=./log.txt # The log file
log.level=info # The runtime log level

# Spaces between the equal sign ARE 
# interpreted.
log.level =debug # "log.level " and "log.level" are different 
log.level= debug # Same goes for this 

```
Currently supported settings are:


| Settings | Description | Required |
|----------|-------------|----------|
| `serial.baud` | Serial baud rate | __Yes__ |
| `serial.device`| Serial device path | __Yes__ |
| `serial.timeout` | Serial timeout in seconds. Default is zero | No |
| `log.file` | Path for logging to a file. | No |
| `log.level` | Run time log level filter. Default is debug (full logging) | No |



 








