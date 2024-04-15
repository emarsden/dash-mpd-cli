# Run safely sandboxed in a Docker container

The application, alongside the external helper applications that it uses for muxing media streams,
for extracting/converting subtitle streams, and for decrypting content infected with DRM, are
available as a prebuilt container, which is probably the easiest and safest way to run it. The container can
be run on any host that can run Linux/AMD64 containers (using [Podman](https://podman.io/) or
[Docker](https://www.docker.com/) on Linux, Microsoft Windows and MacOS, possibly your NAS device).
It’s available in the GitHub Container Registry `ghcr.io` and is automatically built from the sources
using GitHub’s useful continuous integration services.

It’s packaged as a **multiarch container** using the lightweight [Alpine Linux
distribution](https://www.alpinelinux.org/). The following **helper applications** are included in
the container:

- ffmpeg from Alpine Linux

- mkvmerge from Alpine Linux

- mp4decrypt from the Bento4 suite, from Alpine Linux

- MP4Box from the GPAC suite, compiled from source

- xsltproc from the libxslt package, from Alpine Linux

- Shaka packager, from [Google's Docker image](https://hub.docker.com/r/google/shaka-packager) or
  [from GitHub](https://github.com/shaka-project/shaka-packager/releases/latest/), or built from
  source on certain platforms

The container is currently available for the following **platforms**:

- linux/amd64
- linux/arm64/v8
- linux/arm/v7
- linux/riscv64


```admonish info title="Advantages of running in a container"
Why run the application in a container, instead of natively on your machine?

- Good internet hygiene. It’s much safer, because the container is **sandboxed**: it can’t modify
  your host machine, except for writing downloaded media to the directory you specify. This 
  is a very good idea when running random software you downloaded from the internet! The xz
  backdoor attempt discovered in 2024 shows how much effort malicious actors are willing to make to
  compromise peoples’ computers; don’t make it easier than it should be.

- No need to install the various helper applications (ffmpeg, mkvmerge, mp4decrypt, MP4Box),
  which are already present in the container.

- Automatically run the latest version of dash-mpd-cli and the various helper applications (the
  container runtime can pull the latest version for you automatically).

- Podman and Docker also allow you to set various limits on the resources allocated to the
  container (number of CPUs, memory); see their respective documentation.
```

Unlike running software in a virtual machine, there is only a negligeable performance penalty to
running in a container. That’s not quite true: if you’re running the container on an aarch64 (“Apple
Silicon”) Mac, Podman will set up a virtual machine for you. On Windows, Podman will set up a
low-overhead WSL2 virtual machine for you.

```admonish tip
I recommend installing [Podman](https://podman.io/) because it’s fully free software, whereas Docker
is partly commercial. Podman is also able to run containers “rootless”, without special privileges,
which is good for security, and doesn’t require a background daemon. Podman has a docker-compatible
commandline interface.
```


## Running the container

If you’re running on Microsoft Windows or MacOS, you will need to start the virtual machine that’s
used to run the container:

~~~admonish example title="Start up the container runtime (only Windows/MacOS)"
```shell
podman machine start
```

(Replace `podman` by `docker` if you prefer that option.)
~~~

This step is not necessary on Linux.

You can then fetch the container image (currently around 220 MB) from the GitHub container registry
`ghcr.io` and save it to your local disk for later use:

~~~admonish example title="Fetch the container image"
```shell
podman pull ghcr.io/emarsden/dash-mpd-cli
```
~~~

Then to download some content from an MPD manifest:

~~~admonish example title="Run dash-mpd-cli in the container"
```shell
podman run --rm --tty -v .:/content ghcr.io/emarsden/dash-mpd-cli https://example.com/manifest.mpd
```
~~~

This should save the media to a file named something like `example.com_manifest.mp4` 💪 (you can
change this name by adding `-o foo.mp4`). It will remove the container image from your local storage
once the download is finished (`--rm`) and will use a terminal to show a progress bar (`--tty`).

If you want your local copy of the container image to be **updated if a newer one is available** from
the registry, add `--pull=newer`:

```
podman run --rm --tty --pull=newer \
  -v .:/content \
  ghcr.io/emarsden/dash-mpd-cli \
  -v <MPD-URL> -o foo.mp4
```

If you don't use the `--rm` argument, you can later delete the image if you no longer need it using
`podman image rm` with the image id shown by `podman images`, as illustrated below:

~~~admonish example title="Delete the container image from your local disk"
```shell
% podman images
REPOSITORY                       TAG         IMAGE ID      CREATED         SIZE
ghcr.io/emarsden/dash-mpd-cli    latest      ae6971bf21ae  4 days ago      216 MB
...
% podman image rm ae6971bf21ae
```
~~~


## Mounting a directory into the container

By default, your local disk is neither readable nor writable by the application running in the
container (this is a major security advantage!). Since you want to write the downloaded media onto
your local disk, you need to mount (bind) a directory into the container, using podman’s `-v`
commandline option. 

In the commandline show above, your current working directory (`.`) will be mounted in the container
as `/content`, which is always the working directory in the container. This means that an output
file specified without a full path, such as `foo.mp4`, will be saved to your current working
directory on the host machine. If you specify a full path for the output file, for example `-o
/tmp/foo.mp4`, this will output to the temporary directory in the container, which you
won’t have access to once the download has finished.

This sandboxing restriction also applies to any files you need to pass into the container, such as
an XSLT stylesheet for rewriting the manifest. If you’re running podman from your `Videos`
directory, a stylesheet has to be in `Videos` or a subdirectory, or the container won’t be able to
see it. Therefore, you should provide a relative name rather than an absolute name to the container.
If the stylesheet is in the `rewrites` directory, for example:

```shell
podman run --rm --tty --pull=newer \
  -v .:/content \ 
  --xslt-stylesheet rewrites/my-rewrites.xslt \
  ghcr.io/emarsden/dash-mpd-cli \
  -v <MPD-URL> -o foo.mp4
```



## Increased security with gVisor

On Linux/AMD64, it’s also possible to run the container using the [gVisor](https://gvisor.dev/)
container runtime runsc, which uses specially-designed sandboxing techniques to improve security
(strong isolation, protection against privilege escalation). This requires installation of runsc and
running as root (runsc doesn’t currently support rootless operation).

```shell
sudo apt install runsc
sudo podman --runtime=runsc run --rm --tty -v .:/content ghcr.io/emarsden/dash-mpd-cli -v <MPD-URL> -o foo.mp4
```
