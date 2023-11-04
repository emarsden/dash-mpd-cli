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

  <!-- This stylesheet modifies the @media attribute on the SegmentTemplate of the AdaptationSet
       with id=2, setting it to the string '1080p'. This is a way of overriding the DASH template
       resolution process. A template might be using "$RepresentationID$_$Number$.mp4" and you want
       to force the RepresentationID to be 1080p for example.
       
       Note that simple XPath expressions like "//AdaptationSet[@id=2]/SegmentTemplate/@media" fail
       on an MPD manifest that defines a namespace (@xmlns on the MPD element, which is very often
       present). This is why we use local-name().
       -->
  <xsl:template match="//node()[local-name()='AdaptationSet' and @id='2']/node()[local-name()='SegmentTemplate']/@media">
    <xsl:attribute name="media">
      <xsl:value-of select="'1080p_$Number$'"/>
    </xsl:attribute>
  </xsl:template>
</xsl:stylesheet>
