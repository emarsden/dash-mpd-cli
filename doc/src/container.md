# Run safely in a Docker container

The application, alongside the external helper applications that it uses for muxing media streams,
for extracting/converting subtitle streams, and for decrypting content infected with DRM, are
available as a container, which is probably the easiest and safest way to run it. The container is
packaged with a minimal Alpine Linux installation and can be run on any host that can run
Linux/AMD64 containers (using [Podman](https://podman.io/) or [Docker](https://www.docker.com/) on
Linux, Microsoft Windows and MacOS, possibly your NAS device). It’s available in the GitHub
Container Registry ghcr.io and automatically built from the sources using GitHub’s useful continuous
integration services.

What are the advantages of running in a container, instead of natively on your machine?

- Much safer, because the container isn't able to modify your host machine, except for writing
  downloaded media to the directory you specify. This is a very good idea when running random
  software you downloaded from the internet!

- No need to install the various helper applications (ffmpeg, mkvmerge, mp4decrypt, MP4Box),
  which are already present in the container.

- Automatically run the latest version of dash-mpd-cli and the various helper applications (the
  container runtime will pull the latest version for you automatically).

- Podman and Docker also allow you to set various limits on the resources allocated to the
  container (number of CPUs, memory); see their respective documentation.

Unlike running software in a virtual machine, there is only a negligeable performance penalty to
running in a container. That’s not quite true: if you’re running the container on an aarch64 (“Apple
Silicon”) Mac, Podman will set up a virtual machine for you. On Windows, Podman will set up a
low-overhead WSL2 virtual machine for you.

I recommend installing [Podman](https://podman.io/) because it’s fully free software, whereas Docker
is partly commercial. Podman is also able to run containers “rootless”, without special privileges,
which is good for security.


## Running the container

To run the container with podman:

    podman machine start (optional step, only required on Windows and MacOS)
    podman run -v .:/content ghcr.io/emarsden/dash-mpd-cli -v <MPD-URL> -o foo.mp4

(Replace `podman` by `docker` if you prefer that option.)

On the first run, this will fetch the container image (around 216 MB) from the GitHub Container
Registry ghcr.io, and will save it on your local disk for later uses. You can later delete the image
if you not longer need it using `podman image rm` and the image id shown by `podman images`.


## Mounting a directory into the container

By default, your local disk is neither readable nor writable by the application running in the
container (this is a major security advantage!). Since you want to write the downloaded media onto
your local disk, you need to mount (bind) a directory into the container, using podman's `-v`
commandline option. 

In the commandline show above, your current working directory (`.`) will be mounted in the container
as `/content`, which will be the working directory in the container. This means that an output file
specified without a root directory, such as `foo.mp4`, will be saved to your current working
directory on the host machine. If you specify a fully qualified path for the output file, for
example `-o /tmp/foo.mp4`, note that this will output to the temporary directory in the container,
which you won't have access to once the download has finished.

On Linux/AMD64, it’s also possible to run the container using the [gVisor](https://gvisor.dev/)
container runtime runsc, which uses a sandbox to improve security (strong isolation, protection
against privilege escalation). This requires installation of runsc and running as root (runsc
doesn’t currently support rootless operation).

    sudo apt install runsc
    sudo podman --runtime=runsc run -v .:/content ghcr.io/emarsden/dash-mpd-cli -v <MPD-URL> -o foo.mp4
