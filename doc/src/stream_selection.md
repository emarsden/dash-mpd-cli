# Selecting which streams to download

Many DASH manifests offer multiple audio and video streams, with different audio languages,
different video resolutions and bandwidth/quality levels, different codecs, different role labels
(main, alternate, commentary and so on). dash-mpd-cli provides several commandline arguments that
allow you to express preferences for these different attributes, in order to download the audio and
video stream (the Representation, in DASH terminology) you want:

- `--quality` allows you to express a preference for best (highest download size), lowest or
  intermediate quality, that applies both to the video and the audio streams (when they are
  separate). The default behaviour is to prefer the stream with the lowest quality (and lowest
  download size).

- `--prefer-video-width` to request video whose width is closest to the specified number of pixels.

- `--prefer-video-height` to request video whose height is closest to the specified number of pixels.

- `--prefer-video-codecs` to specify which video codecs to prefer, for multi-codec manifests where
  the same video content is available in the same resolution but using different encoding methods.
  This option takes a comma-separated list of the form `avc1,hev1,vvc1` in which each codec is
  specified in FourCC format. You can see the video codecs which are available for a manifest by
  using the `--simulate` commandline option (if full `family.subfamily` codec names are specified,
  you can use only the family part of the name).

- `--want-video-id` to specify which video Representation to download by its id. The provided
  substring is used as a filter on available video Representations: if the full id is provided this
  selects the specified Representation, and if only a substring of the id is specified, this
  preference will be combined with other preferences such as the quality level and codec preference
  to select a single preferred video stream. Use the `--simulate` commandline option to see the ids
  available in a manifest.

- `--prefer-language` to request the audio track with the desired language.

- `--role-preference` allows you to specify a preference ordering for the Role label present in
  certain manifests. The default behaviour is to prefer a stream labelled with a `main` role over an
  `alternate` role.

If you run dash-mpd-cli with the `--verbose` and `--simulate` arguments, it will print information
on the attributes of the available streams, similar to that shown below:

```
10:45:17  INFO Streams in period #1, duration 12m 14s:
10:45:17  INFO   audio mp4a.40.2         |    62 Kbps |   lang=en role=main (id=audio_eng=64349)
10:45:17  INFO   audio mp4a.40.2         |   125 Kbps |   lang=en role=main (id=audio_eng=128407)
10:45:17  INFO   video hev1.1.6.H150.90  |  1007 Kbps |  1680x750 role=main (id=video_eng=1032000)
10:45:17  INFO   video hev1.1.6.H150.90  |  1220 Kbps | 2576x1150 role=main (id=video_eng=1250000)
10:45:17  INFO   video hev1.1.6.H150.90  |  1562 Kbps | 3360x1500 role=main (id=video_eng=1600000)
10:45:17  INFO   subs          Wvtt/wvtt |         en | role=subtitle
10:45:17  INFO   subs          Wvtt/wvtt |         de | role=subtitle
10:45:17  INFO   subs          Wvtt/wvtt |         es | role=subtitle
10:45:17  INFO   subs          Wvtt/wvtt |         fr | role=subtitle
10:45:17  INFO   subs          Wvtt/wvtt |         nl | role=subtitle
10:45:17  INFO   subs          Wvtt/wvtt |      pt-br | role=subtitle
10:45:17  INFO   subs          Wvtt/wvtt |      pt-pt | role=subtitle
10:45:17  INFO   subs          Wvtt/wvtt |         ru | role=subtitle
10:45:17  INFO   subs          Wvtt/wvtt |         zh | role=subtitle
10:45:17  INFO   subs          Wvtt/wvtt |    zh-hans | role=subtitle
```

The list belows specifies the **order** in which these preferences are handled:

- First filter out AdaptationSets in the manifest that do not correspond to our language
  preference. If not language preference is specified, no filtering takes place. If multiple
  AdaptationSets match the language preference, they all are passed on to the next stage of
  filtering.

- Select adaptations according to the role preference. If no role preference is specified, no
  filtering takes place based on the role labels. If no adaptations match one of our role
  preferences, no filtering takes place based on the role labels. If at least one adaptation matches
  one role in the expressed role preference, only the adaptation which is closest to the head of the
  role preference list is passed on to the next stage of filtering.

- When multiple Representation elements are present, filter them according to the video id
  substring, if specified. If the full id is provided this selects the specified Representation, and
  if only a substring of the id is specified, all Representations that match the substring move to
  the next stage of filtering.

- If a video width preference is specified, retain only the Representations with a video width
  closest to the requested width (there may be multiple Representations with the same video width
  but with different codecs, for example).

- If a video height preference is specified, retain only the Representations with a video height
  closest to the requested height.

- When multiple Representation elements are still present, filter them according to any specified
  quality preference. If no quality preference is specified, the Representation (audio and video)
  with the lowest quality/bandwidth (and therefore file size) is selected. The filtering is based on
  the `@qualityRanking` attribute, if it is specified on the Representation elements, and otherwise
  based on the `@bandwidth` attribute specified. Note that quality ranking may be different from
  bandwidth ranking when different codecs are used.

- If more than one stream remains under consideration after all the preceding steps, select the
  first stream that appears in the XML of the DASH manifest.
