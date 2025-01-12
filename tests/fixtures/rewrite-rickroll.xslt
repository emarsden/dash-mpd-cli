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
      This stylesheet modifies the @initialization and @media attribute on SegmentTemplate elements,
      as well as the content of BaseURL elements, to point to a beloved media segment. It also drops
      the audio AdaptationSet, for obvious mental health reasons.

      To test this stylesheet:

      xsltproc rewrite-rickroll.xslt input.mpd
  -->

  <xsl:template match="//mpd:BaseURL">
    <BaseURL>https://dash.akamaized.net/akamai/test/rick_dash_track1_init.mp4</BaseURL>
  </xsl:template>

  <!-- delete any indexRange attributes on a SegmentBase node, because our replacement media segment
  is not set up with the same index -->
  <xsl:template match="//mpd:SegmentBase/@indexRange" />

  <xsl:template match="//mpd:SegmentTemplate/@initialization">
    <xsl:attribute name="initialization">
      <xsl:value-of select="'https://dash.akamaized.net/akamai/test/rick_dash_track1_init.mp4'"/>
    </xsl:attribute>
  </xsl:template>

  <xsl:template match="//mpd:SegmentTemplate/@media">
    <xsl:attribute name="media">
      <xsl:value-of select="'https://dash.akamaized.net/akamai/test/rick_dash_track1_init.mp4'"/>
    </xsl:attribute>
  </xsl:template>

  <xsl:template match="//mpd:AdaptationSet[@contentType='audio']"/>
  <xsl:template match="//mpd:AdaptationSet[starts-with(@mimeType, 'audio/')]"/>
</xsl:stylesheet>
