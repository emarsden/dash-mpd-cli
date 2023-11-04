<?xml version="1.0" encoding="utf-8"?>
<xsl:stylesheet version="1.0"
                xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
                xmlns:mpd="urn:mpeg:dash:schema:mpd:2011">
  <xsl:output method="xml" indent="yes"/>

  <!-- Default action (unless a template below matches): copy -->
  <xsl:template match="@*|node()">
    <xsl:copy>
      <xsl:apply-templates select="@*|node()"/>
    </xsl:copy>
  </xsl:template>

  <!--
      Drop Periods served from dai.google.com or from AWS MediaTailor or from Unified Streaming
      (dynamic ad insertion services).

      Test: xsltproc rewrite-drop-dai.xslt with-dai.mpd
  -->
  <xsl:template match="//mpd:Period[mpd:BaseURL[starts-with(text(),'https://dai.google.com')]]" />

  <xsl:template match="//mpd:Period[mpd:BaseURL[contains(text(),'mediatailor.eu-west-1.amazonaws.com')]]" />

  <xsl:template match="//mpd:Period[mpd:BaseURL[contains(text(),'unified-streaming.com')]]" />
</xsl:stylesheet>
