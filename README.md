# `lan-party-tools`

A command line utility to make organizing for LAN Parties a bit easier.

> I'm building this as I learn Rust, so the code quality is probably terrible...

## Download

Get the latest released version [here](https://github.com/Steffo99/lan-party-tools/releases).

## Usage

### View help information

```
lan-party-tools --help
```

### Get information about the current network(s)

```
lan-party-tools network
```

### List all installed Steam games

```
lan-party-tools steam list
```

If you are using a non-default steamapps directory, you can specify it with the `--steamapps DIRECTORY` option.

### Backup Steam games to an external hard drive

```
lan-party-tools steam backup {APPID}+
```

The games are backed up to the current working directory. If you want to specify a different directory, use the `--destination DIRECTORY` option.

If you are using a non-default steamapps directory, you can specify it with the `--steamapps DIRECTORY` option.

### Restore Steam games from a previous backup

```
lan-party-tools steam restore {APPID}+
```

The games are restored from the current working directory. If you want to specify a different directory, use the `--source DIRECTORY` option.

If you are using a non-default steamapps directory, you can specify it with the `--steamapps DIRECTORY` option.
