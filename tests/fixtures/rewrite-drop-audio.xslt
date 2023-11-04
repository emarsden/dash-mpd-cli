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
      Drop any audio/* AdaptationSets, leaving only the AdaptationSets with mimeType of video/mp4.
      
      Note that in principle we should be able to write the XPath expression we are matching on in
      this simpler form

         "//mpd:AdaptationSet[starts-with(@mimeType, 'audio/')]"

      but unfortunately this manifest is using an incorrectly capitalized xmlns declaration
      xmlns="urn:mpeg:DASH:schema:MPD:2011". XML is case sensitive; this namespace is not the same
      as the one specified in the DASH specifications (this error is infrequent but is present in
      the wild).
  -->
  <xsl:template match="//node()[local-name()='AdaptationSet' and starts-with(@mimeType,'audio/')]" />
</xsl:stylesheet>
