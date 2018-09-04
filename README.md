## Installation

1. download latest package file from release page.
2. run command `$ sudo pkg add {{package file}}`.
3. run command `$ sudo sysrc jmaker_local_network_agent_enable=YES`.
4. and run service in background `service jmaker-local-network-agent start`.

## Configuration

- see `/usr/local/etc/rc.d/jmaker-local-network-agent` for configuration service
- run `$ jmaker-local-network-agent --help`

