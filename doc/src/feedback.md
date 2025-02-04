# Your feedback

The project [discussions page](https://github.com/emarsden/dash-mpd-cli/discussions) is a good place
to ask questions regarding the use of dash-mpd-cli.

Bug reports should be filed as issues on [our GitHub project
page](https://github.com/emarsden/dash-mpd-cli). 

Pull requests are also welcome!



## Debugging tips

dash-mpd-cli will show information about its ongoing activity if you use the `--verbose` (or `-v`)
commandline argument. You can use this up to three times for increasing verbosity. At level 3, each
network request made to download a media segment will be shown in the logs.

To obtain additional information, you can set the `RUST_LOG` environment variable. For example, a
value of

    debug,reqwest=trace,hyper=trace,h2=trace

means to log for most software libraries used by dash-mpd-cli at level `debug`, and at `trace` level
(which is more verbose) for the libraries `reqwest`, `hyper` and `h2` (these are all used for
network requests). For more information on this fairly powerful logging functionality, see the
[documentation of the EnvFilter
module](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)
of the `tracing_subscriber` crate.

This may be useful if you want to work out whether cookies from your browser are being sent with
network requests, or if you need to debug a TLS connection problem.



~~~admonish example title="Run with a temporary value for RUST_LOG"

**On Linux**: to run dash-mpd-cli with a temporary value for RUST_LOG, without changing the value of that
environment variable globally, you can say (with most shells):

```shell
RUST_LOG=debug,reqwest=trace,hyper=trace,h2=trace dash-mpd-cli -v -v -v https://example.com/manifest.mpd
```

**On Microsoft Windows**: something like the following may work:

```shell
set "RUST_LOG=debug,reqwest=trace,hyper=trace,h2=trace" & dash-mpd-cli -v -v -v https://example.com/manifest.mpd
```

If you are running dash-mpd-cli in a Docker/Podman [container](container.html), pass the environment
variable using the `-e` commandline argument to docker/podman.

~~~

