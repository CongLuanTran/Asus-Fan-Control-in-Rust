# Why do I need this program?

This reposistory's main purpose is to show how I control my asus laptop fan when all other methods failed. To be clear my fan is still working, but its speed is not what I want, it speeds up too slow. Hopefully you shouldn't need this one. I belive that usually fan control can be achieved with other means as suggested in the Arch Linux wiki [page](https://wiki.archlinux.org/title/Fan_speed_control).

On asus laptops, there is 'asus-nb-wmi', a kernel module that can control one fan. According to the Arch wiki page linked above, there is a file called 'pwm1' to control the speed of the fan, but in my system that file doesn't exist. Another way is to turn pwm mode on or off by writing a value into the file 'pwm1_enable'. The default value is '2', which is the default mode of the fan. Set it to '0' will turn the fan on at full speed, while '1' will shut it down.

The idea of this program is to read CPU temperature , then if it reach some threshold, we turn the fan on at full speed, else we return it to default mode. More details are in the comment inside the 'main.rs' file More details are in the comment inside the 'main.rs' file.

# Dependencies

This program need the rust toolchain to compile, so you should download `rustup` from the [official website](https://rustup.rs/).

# Installation

The easiest way should be to clone this reposistory to whatever folder you like, then `cd` into it and run. You can also read the INSTALL.sh to know more about what it does.

```bash
chmod +x INSTALL.sh
./INSTALL.sh
```
# TODO

- Make the code more modular