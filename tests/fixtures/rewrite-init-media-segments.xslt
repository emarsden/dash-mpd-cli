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
      This stylesheet modifies the @initialization and @media attribute on the SegmentTemplate of
      the video AdaptationSet. It also drops the audio AdaptationSet. It also modifies the BaseURL
      to point to localhost.

      This stylesheet is used by one of the tests in file test/xslt.rs.

      To test this stylesheet:

      xsltproc rewrite-init-media-segments.xslt input.mpd
  -->
  <xsl:template match="/mpd:MPD/mpd:BaseURL">
    <BaseURL>http://localhost:6668/</BaseURL>
  </xsl:template>

  <xsl:template match="//mpd:AdaptationSet[@contentType='video']/mpd:SegmentTemplate/@initialization">
    <xsl:attribute name="initialization">
      <xsl:value-of select="'/media/init.mp4'"/>
    </xsl:attribute>
  </xsl:template>

  <xsl:template match="//mpd:AdaptationSet[@contentType='video']/mpd:SegmentTemplate/@media">
    <xsl:attribute name="media">
      <xsl:value-of select="'/media/segment-$Number$.mp4'"/>
    </xsl:attribute>
  </xsl:template>

  <xsl:template match="//mpd:AdaptationSet[@contentType='audio']"/>
</xsl:stylesheet>
