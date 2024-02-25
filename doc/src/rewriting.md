# Rewriting the MPD manifest

For advanced users, there is some experimental support for rewriting the MPD manifest before
downloading media segments from it. This allows you to:

- print additional diagnostics concerning the manifest, which aren’t printed by dash-mpd-cli even
  with a high verbose level.

- delete some Periods that the user is not interested in (based for example on their duration, or
  the origin of the media segments). This can be used to remove advertising segments inserted using
  dynamic ad insertion (DAI) or server-side ad insertion (SSAI) techniques. 

- delete from the manifest Representations that use undesired codecs. This is a way of making the
  choice of representation fall back to another Representation, which presumably uses an
  acceptable codec.
    
- delete audio Representations whose language the user is not interested in (though in this case
  dash-mpd-cli has a builtin mechanism with `--prefer-language` to select the desired
  Representation).
  
- delete all subtitle languages and formats except for the one the user is interested in (again, as
  a complement to the builtin `--prefer-language` functionality).
  
- drop an audio AdaptationSet if the user only is interested in video (though this functionality is
  already builtin with `--video-only`).

- modify the BaseURL to include another CDN.

You can use this functionality by:

- supplying an [XPath expression](https://en.wikipedia.org/wiki/XPath) matching XML elements that
  you don’t want to download, with the `--drop-elements` commandline option;
  
- supplying a full [XSLT](https://en.wikipedia.org/wiki/XSLT) stylesheet that will be applied to the
  manifest to allow more complex rewriting rules. XSLT is a language that is specifically designed
  for XML filtering/rewriting; it’s standards-based though not particularly intuitive.

This functionality is currently implemented by calling out to the
[xsltproc](http://xmlsoft.org/xslt/xsltproc.html) commandline application, which supports XPath v1.0
and XSLT v1.0.


## Examples of XPath filtering

~~~admonish example title="Drop AdaptationSets with alternate audio"

Suppose your DASH manifest contains an audio track with audio description, which has an attribute
`label=alternate`. This track is being selected for download instead of the one you want. You can
filter out the audio description track using the `--drop-elements` commandline argument with the
following XPath expression:

```
--drop-elements "//mpd:AdaptationSet[@label='alternate']
```

If instead the audio track is marked with a `Label` element that contains the text `audiodescr`, you
can use the following:

```
--drop-elements "//mpd:AdaptationSet[.//mpd:Label[contains(text(), 'audiodescr')]]"
```

If instead the AdaptationSet you want to ignore is identified by a `Role` child element, something
like

```
<Role schemeIdUri="urn:mpeg:dash:role:2011" value="alternate"/>
```

you can use the following

```
--drop-elements "//mpd:AdaptationSet[.//mpd:Role[@value='alternate']]"
```

~~~

Another possible application of the XML filtering capability is to avoid overloading the web servers
of companies that serve dynamic ad insertion (DAI) content, as illustrated below.


~~~admonish example title="Drop dynamically inserted advertising content"

Some DASH manifests include content (generally some `Period` elements) which is inserted based on
your prior viewing habits, the time of day, your geographic location, and so on. You may wish to
filter these out based on the URL of the server, with for example

```
--drop-elements "//mpd:Period[mpd:BaseURL[contains(text(),'https://dai.google.com')]]
--drop-elements "//mpd:Period[mpd:BaseURL[contains(text(),'mediatailor.eu-west-1.amazonaws.com')]]"
--drop-elements "//mpd:Period[mpd:BaseURL[contains(text(),'unified-streaming.com')]]"
```

or based on other features such as the Period duration or keywords in the BaseURL:

```
--drop-elements "//mpd:Period[duration='PT5.000S']"
--drop-elements "//mpd:Period[.//mpd:BaseURL[contains(text(),'/creative/')]]"
--drop-elements "//mpd:Period[.//mpd:BaseURL[contains(text(),'Ad_Bumper')]]"
```
~~~



## Examples of XSLT rewriting

The XSLT file (stylesheet) shown below will drop any AdaptationSets in the MPD manifest with a
`@mimeType` matching `audio/*` (leaving only the AdaptationSets containing video).

```xml
{{#include ../../tests/fixtures/rewrite-drop-audio.xslt}}
```

Note that the rewriting instruction 

```xml
  <!-- Default action (unless a template below matches): copy -->
  <xsl:template match="@*|node()">
    <xsl:copy>
      <xsl:apply-templates select="@*|node()"/>
    </xsl:copy>
  </xsl:template>
```

acts as a default action that will copy verbatim to the output any XML elements that aren’t matched
by another template in the stylesheet.

The rewriting instruction 

```xml
<xsl:template match="//node()[local-name()='AdaptationSet' and starts-with(@mimeType,'audio/')]" />
```

is selecting (using the XPath expression defined in the template’s `@match` attribute) all
AdaptationSet nodes whose `@mimeType` attribute starts with `audio/`. It doesn’t specify any action
to run on these elements, which means that they are not copied to the XML output.

To run an XSLT template, see the `--xslt-stylesheet` commandline argument. There are a few example
of stylesheets in the [tests/fixtures
directory](https://github.com/emarsden/dash-mpd-cli/tree/main/tests/fixtures).

To download content with a rewritten manifest (here [running dash-mpd-cli in a container](container.html)):

    podman run --rm -v .:/content \
      --xslt-stylesheet my-rewrites.xslt \
      ghcr.io/emarsden/dash-mpd-cli \
      https://example.com/manifest.mpd -o foo.mp4



## Future plans

Our current implementation of filtering using xsltproc is quite powerful and easy to install, but
probably not the easiest to use. Possible alternatives which we might move to in future version of
dash-mpd-cli: 

- Saxon-HE, free Java software (MPL v2) which implements XPath v3.1 and XSLT v3.0

- A generic filter interface implemented as a pipe

- A command API that takes two filename arguments

- A WebAssembly-based interface that could be implemented in any programming language that can
  generate WASM bytecode.


