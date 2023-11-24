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

This functionality is currently implemented using [XSLT](https://en.wikipedia.org/wiki/XSLT), a
language developed for XML rewriting. This is a standards-based approach to filtering/rewriting,
which is very powerful though not particularly intuitive not very widely adopted. XSLT is
implemented by calling out to the xsltproc commandline application, which unfortunately only
supports XSLT v1.0. Version 3.0 of the specification is more powerful, and for example includes
functions for manipulating xs:duration attributes which can be useful for our purposes, but the
only free implementation of XSLT 3.0 is implemented in Java and inconvenient to package.


## Examples

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

To run an XSLT template, see the `--xslt-stylesheet` commandline argument.

The XSLT stylesheet shown below will drop any Period elements in the MPD manifest that are served
from dai.google.com or from AWS MediaTailor or from Unified Streaming. These are some of the main
dynamic ad insertion services, which insert ads based on your prior viewing habits, the time of day,
your geographic location, and so on.

```xml
{{#include ../../tests/fixtures/rewrite-drop-dai.xslt}}
```
The important parts of the stylesheet are the XPath expression that select the Period elements to be
dropped, such as 

    //mpd:Period[mpd:BaseURL[contains(text(),'mediatailor.eu-west-1.amazonaws.com')]]

You can adapt these and add additional templates for advertising services used by your telecoms
provider or streaming service.



## Future plans

Our current implementation of filtering using xsltproc is quite powerful and easy to install, but
probably not the easiest to use. Possible alternatives which we might move to in future version of
dash-mpd-cli: 

- Saxon-HE, free Java software (MPL v2) which implements XSLT 3.0

- A generic filter interface implemented as a pipe

- A command API that takes two filename arguments

- A WebAssembly-based interface that could be implemented in any programming language that can
  generate WASM bytecode.





