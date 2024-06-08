# Selecting which streams to download

Many DASH manifests offer multiple audio and video streams, with different audio languages,
different video resolutions and bandwidth/quality levels, different codecs, different role labels
(main, alternate, commentary and so on). dash-mpd-cli provides several commandline arguments that
allow you to express preferences for these different attributes, in order to download the audio and
video stream you want:

- `--quality` allows you to express a preference for best (highest download size), lowest or
  intermediate quality, that applies both to the video and the audio streams (when they are
  separate). The default behaviour is to prefer the stream with the lowest quality (and lowest
  download size).
  
- `--prefer-video-width` to request video whose width is closest to the specified number of pixels.

- `--prefer-video-height` to request video whose height is closest to the specified number of pixels.
  
- `--prefer-language` to request the audio track with the desired language.

- `--role-preference` allows you to specify a preference ordering for the Role label present in
  certain manifests. The default behaviour is to prefer a stream labelled with a `main` role over an
  `alternate` role.

If you run dash-mpd-cli with the `--verbose` and `--simulate` arguments, it will print information
on the attributes of the available streams, similar to that shown below:

```
07:59:26  INFO Streams in period p0 (#1), duration 464.000s:
07:59:26  INFO   audio mp4a.40.2         |    94 Kbps |  lang=eng role=main
07:59:26  INFO   audio mp4a.40.2         |   126 Kbps |  lang=eng role=main
07:59:26  INFO   audio mp4a.40.2         |    94 Kbps |  lang=fin role=alternate
07:59:26  INFO   audio mp4a.40.2         |   126 Kbps |  lang=fin role=alternate
07:59:26  INFO   audio mp4a.40.2         |    94 Kbps |  lang=ger role=alternate
07:59:26  INFO   audio mp4a.40.2         |   126 Kbps |  lang=ger role=alternate
07:59:26  INFO   audio mp4a.40.2         |    94 Kbps |  lang=swe role=alternate
07:59:26  INFO   audio mp4a.40.2         |   126 Kbps |  lang=swe role=alternate
07:59:26  INFO   video hvc1.1.6.L120.90  |   469 Kbps |  1280x720 role=main
07:59:26  INFO   video hvc1.1.6.L120.90  |   791 Kbps | 1920x1080 role=main
07:59:26  INFO   video hvc1.1.6.L150.90  |  1503 Kbps | 3840x2160 role=main
```

The list belows specifies the order in which these preferences are handled:

- First filter out AdaptationSets in the manifest that do not correspond to our language
  preference. If not language preference is specified, no filtering takes place. If multiple
  AdaptationSets match the language preference, they all are passed on to the next stage of
  filtering.

- Select adaptations according to the role preference. If no role preference is specified, no
  filtering takes place based on the role labels. If no adaptations match one of our role
  preferences, no filtering takes place based on the role labels. If at least one adaptation matches
  one role in the expressed role preference, only the adaptation which is closest to the head of the
  role preference list is passed on to the next stage of filtering.

- When multiple Representation elements are present, filter them according to any specified quality
  preference. If no quality preference is specified, no filtering takes place. The filtering is
  based on the `@qualityRanking` attribute, if it is specified on the Representation elements, and
  otherwise based on the `@bandwidth` attribute specified. Note that quality ranking may be
  different from bandwidth ranking when different codecs are used.

- If a video width preference is specified, only select the Representation whose video width is
  closest to the requested width.

- If a video height preference is specified, only select the Representation whose video height is
  closest to the requested height.

- If more than one stream remains under consideration after all the preceding steps, select the
  first stream that appears in the XML of the DASH manifest.


