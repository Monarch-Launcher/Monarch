# Steam login

## Basics
In the settings menu you can add your username and passward. These are stored in your operating systems secure store, and are *never* sent to a remote server by Monarch. If you are interested in the actual code you can check out `src-tauri/src/monarch_utils/monarch_credentials.rs`. 

## SteamGuard
If you have SteamGuard enabled on your account you will be prompted to put in the SteamGuard code in the terminal when running any SteamCMD command. There are two ways of circumventing this. 

1. Disable SteamGuard. A pretty straight-forward way to get around the SteamGuard issue. This is however a security risk as you now no longer have 2FA.
2. Replace your existing SteamGuard. Below is a guide for replacing your existing SteamGuard with a Monarch compatible version. **WARNING:** This is also a security risk as Monarch will be able to calculate your SteamGuard code for you. If anyone manages to get access to your machine both your login and 2FA is compromised. You have been warned, proceed with caution.

## Replacing SteamGuard
**This guide is inspired by:** https://gist.github.com/mathielo/8367e464baa73941a075bae4dd5eed90
Be sure to read the disclaimers on there to understand potential consequences.

### Summary
This guide will help you replace your existing SteamGuard solution with a normal 2FA for Steam. The point of doing this is that it enables Monarch to automatically produce your SteamGuard code. This is done by getting your *shared secret* used to produce the code.

### How does Monarch do this?
Via a forked crate that is fully open source at: https://github.com/Monarch-Launcher/simple-steam-totp
Big thanks to [Weilbyte](https://github.com/Weilbyte) who made the original.

### Guide
First of all, disable your current SteamGuard. This can be done using Steams official instructions:
https://store.steampowered.com/twofactor/remove?step=ondevice  

The original guide suggests generating your new shared secret via https://github.com/ValvePython/steam.
However it didn't work for me, just like the comments. One commenter wrote a suggestion which *did* work for me, using https://github.com/dyc3/steamguard-cli.  

First download the correct version for your operating system, either by compiling from source, or using a pre-compiled source from https://github.com/dyc3/steamguard-cli/releases.

The shared secret can then be generated with the following command:
```
steamguard setup
```

The shared secret can then be put into an authenticator app, such as Bitwarden authenticator ([info](https://www.reddit.com/r/Bitwarden/comments/t9xbkp/til_you_can_use_bitwarden_2fa_for_steam/)), to be used as a normal 2FA. You can also copy the shared secret into the Steam Shared Secret field in the Monarch settings.
