# mini-tools
A collection of small tools 🙂

| Binaries | Version |
| :------- | :------ |
| Toaster  | 0.1     |

### Toaster
An easy way to display Toast notifications

Note: It can fail the first time, if the start menu is not updated

Usage:
  ```toaster.exe [FLAGS] [OPTIONS]```

##### Flags:
```
-n, --noshortcut    Toaster doesn't try to create it's a shortcut in start menu (userspace)
    --help          Prints help information
-V, --version       Prints version information
```
##### Options:
```
-a, --appid <AppID>          The location the program you want to display the message. The program must have a
                             shortcut in the start menu
-f, --file <File>            Uses an XML file as the Toast
-h, --headline <Headline>    Sets the headline of the Toast
-i, --icon <IconPath>        Sets the icon of the Toast
-t, --text <Text>            Sets the message of the Toast
```
