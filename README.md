# rendrok

`rendrok` is a utility to make [chisel](https://github.com/jpillora/chisel) tunnels on [render.com](https://render.com).

## Get Started

If you don't have `chisel` installed already, it has an one-line script to [help you out](https://github.com/jpillora/chisel).

Click [here](https://render.com/deploy?repo=https://github.com/evshiron/rendrok) to deploy [chisel](https://github.com/jpillora/chisel) server on [render.com](https://render.com) with generated password, which `rendrok` will obtain via [render.com](https://render.com) API.

Go to [render.com Dashboard](https://dashboard.render.com/) -> Account Settings -> API Keys, and create a new API key for rendrok use.

```bash
# install rendrok via cargo install
cargo install --git https://github.com/evshiron/rendrok
```

## Usage

```
Usage: rendrok <COMMAND>

Commands:
  login   Log in with render.com api key
  ls      List existing rendrok services
  rm      Remove a rendrok service
  serve   Serve a port via rendrok service
  logout  Log out and clean up config
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help information
```

```bash
# log in with your render.com api key
rendrok login

# expose 127.0.0.1:3000 to deployed rendrok service
rendrok serve 3000

# expose 192.168.1.100:3000 to deployed rendrok service
rendrok serve --host 192.168.1.100 3000
```

## Disclaimer

We have nothing to do with Render, or Render Inc.

`rendrok` is built to make life easier.
Use `rendrok` on your own risk, and do not violate their Terms of Service.

## Acknowledgement

* [render.com](https://render.com) for excellent free services
* [chisel](https://github.com/jpillora/chisel) for websocket tunnel implementation

## License

AGPL-3.0
