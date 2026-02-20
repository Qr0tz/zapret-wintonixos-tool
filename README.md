# Port strategies from Flowseal/zapret-discord-youtube repo to nixos config
One-click utility for downloading and porting strategies from Flowseal/zapret-discord-youtube repo to your nixos configuration (zapret nixos service configurations)

## Process:
1. Tool downloads necessary files from Flowseal/zapret-discord-youtube repo to /etc/zapretfiles
2. Converts to .nix configuration files (layout below)
```nix
{ config, pkgs, ...}:

{
  services.zapret = {
    enable = true;
    udpSupport = true;
    udpPorts = [ "443" "1024-65535" ];
    
    params = [
      params_from_config
    ];
  };
}
```
3. It will try to copy files into /etc/nixos/zapret
4. Done!

# Thanks for visiting
