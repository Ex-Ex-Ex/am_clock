#AM Clock tool

![Stars](https://img.shields.io/github/stars/ex-ex-ex/am_clock) ![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FEx-Ex-Ex%2Fam_clock%2Fmain%2FCargo.toml&query=package.version&label=Version)

Small utility to update the clock of Angry Miao Cyberboards to the PC's local time

## Usage

The utility is a small cli (terminal) program written in rust that  silently updates the clock of a connected Cyberboard to the PC's local time. As an added feature, the tool can set the time to a fake AM / PM mode - there is no "am" / "pm" indicator, and the tool has to be started every day at midnight and midday to adjust the Cyberboard's time. I considered a system daemon to automate that, but I feel that's something better handled by the user's system task scheduler.

### Options

| Parameter | Description |
| ------------- | ------------------------------ |
| `--help, -h`| Display the command line options |
| `--version, -V`| Show version information |
| `-v`| Print status to the console (do this if the utility doesn't work) |
| `-a`| AM/PM mode (instead of the default 24h mode) |

## Credits

All the hard work was done by [Evangelos Ch.](https://gist.github.com/evangelos-ch) who solved this problem 2 years ago, but used python. See the original [gist](https://gist.github.com/evangelos-ch/79b5508dc6d14c7f4e2414fa9bc12a05).
